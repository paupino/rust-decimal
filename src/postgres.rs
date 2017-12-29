extern crate byteorder;
extern crate num;

use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use self::num::{One, Zero, ToPrimitive};
use super::Decimal;
use pg_crate::types::*;
use std::error;
use std::fmt;
use std::io::Cursor;
use std::result::*;

#[derive(Debug, Clone, Copy)]
pub struct InvalidDecimal;

impl fmt::Display for InvalidDecimal {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(error::Error::description(self))
    }
}

impl error::Error for InvalidDecimal {
    fn description(&self) -> &str {
        "Invalid Decimal"
    }
}

impl FromSql for Decimal {
    // Decimals are represented as follows:
    // Header:
    //  u16 numGroups
    //  i16 weightFirstGroup (10000^weight)
    //  u16 sign (0x0000 = positive, 0x4000 = negative, 0xC000 = NaN)
    //  i16 dscale. Number of digits (in base 10) to print after decimal separator
    //
    //  Psuedo code :
    //  const Decimals [
    //          0.0000000000000000000000000001,
    //          0.000000000000000000000001,
    //          0.00000000000000000001,
    //          0.0000000000000001,
    //          0.000000000001,
    //          0.00000001,
    //          0.0001,
    //          1,
    //          10000,
    //          100000000,
    //          1000000000000,
    //          10000000000000000,
    //          100000000000000000000,
    //          1000000000000000000000000,
    //          10000000000000000000000000000
    //  ]
    //  overflow = false
    //  result = 0
    //  for i = 0, weight = weightFirstGroup + 7; i < numGroups; i++, weight--
    //    group = read.u16
    //    if weight < 0 or weight > MaxNum
    //       overflow = true
    //    else
    //       result += Decimals[weight] * group
    //  sign == 0x4000 ? -result : result

    // So if we were to take the number: 3950.123456
    //
    //  Stored on Disk:
    //    00 03 00 00 00 00 00 06 0F 6E 04 D2 15 E0
    //
    //  Number of groups: 00 03
    //  Weight of first group: 00 00
    //  Sign: 00 00
    //  DScale: 00 06
    //
    // 0F 6E = 3950
    //   result = result + 3950 * 1;
    // 04 D2 = 1234
    //   result = result + 1234 * 0.0001;
    // 15 E0 = 5600
    //   result = result + 5600 * 0.00000001;
    //

    fn from_sql(_: &Type, raw: &[u8]) -> Result<Decimal, Box<error::Error + 'static + Sync + Send>> {
        let mut raw = Cursor::new(raw);
        let num_groups = raw.read_u16::<BigEndian>()?;
        let weight = raw.read_i16::<BigEndian>()?; // 10000^weight
        // Sign: 0x0000 = positive, 0x4000 = negative, 0xC000 = NaN
        let sign = raw.read_u16::<BigEndian>()?;
        // Number of digits (in base 10) to print after decimal separator
        let fixed_scale = raw.read_u16::<BigEndian>()? as i32;

        // Build up a list of powers that will be used
        let mut powers = vec![Decimal::one()];
        let mut val = Decimal::one();
        let mult = Decimal::new(10000, 0);
        for _ in 1..num_groups {
            val *= mult;
            powers.push(val);
        }
        powers.reverse();

        // Now process the number
        let mut result = Decimal::zero();
        for i in 0..num_groups {
            let group = raw.read_u16::<BigEndian>()?;
            let calculated = &powers[i as usize] * Decimal::new(group as i64, 0);
            result = result + calculated;
        }

        // Finally, adjust for the scale
        let mut scale = (num_groups as i16 - weight - 1) as i32 * 4;
        // Scale could be negative
        if scale < 0 {
            result *= Decimal::new(10i64.pow((scale * -1) as u32), 0);
            scale = 0;
        } else if scale > fixed_scale {
            result /= Decimal::new(10i64.pow((scale - fixed_scale) as u32), 0);
            scale = fixed_scale;
        }

        // Create the decimal
        let neg = sign == 0x4000;
        if result.set_scale(scale as u32).is_err() {
            return Err(Box::new(InvalidDecimal));
        }
        result.set_sign(!neg);

        // Normalize by truncating any trailing 0's from the decimal representation
        Ok(result.normalize())
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            NUMERIC => true,
            _ => false,
        }
    }
}

impl ToSql for Decimal {
    fn to_sql(&self, _: &Type, out: &mut Vec<u8>) -> Result<IsNull, Box<error::Error + 'static + Sync + Send>> {
        let sign = if self.is_sign_negative() { 0x4000 } else { 0x0000 };
        let scale = self.scale() as u16;

        let mut whole = *self;
        whole.set_scale(0).ok();
        whole.set_sign(true);
        let mut digits = whole.to_string();
        let split_point = if scale as usize > digits.len() {
            let mut new_digits = vec!['0'; scale as usize - digits.len() as usize];
            new_digits.extend(digits.chars());
            digits = new_digits.into_iter().collect::<String>();
            0
        } else {
            digits.len() as isize - scale as isize
        };
        let (whole_digits, decimal_digits) = digits.split_at(split_point as usize);
        let whole_portion = whole_digits
            .chars()
            .rev()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|x| {
                let mut x = x.to_owned();
                while x.len() < 4 {
                    x.push('0');
                }
                x.into_iter().rev().collect::<String>()
            })
            .rev()
            .collect::<Vec<String>>();
        let decimal_portion = decimal_digits
            .chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|x| {
                let mut x = x.to_owned();
                while x.len() < 4 {
                    x.push('0');
                }
                x.into_iter().collect::<String>()
            })
            .collect::<Vec<String>>();
        let weight = if whole_portion.is_empty() {
            -(decimal_portion.len() as i16)
        } else {
            whole_portion.len() as i16 - 1
        };
        let all_groups = whole_portion
            .into_iter()
            .chain(decimal_portion.into_iter())
            .skip_while(|ref x| *x == "0000")
            .collect::<Vec<String>>();
        let num_groups = all_groups.len() as u16;

        // Number of groups
        out.write_u16::<BigEndian>(num_groups)?;
        // Weight of first group
        out.write_i16::<BigEndian>(weight)?;
        // Sign
        out.write_u16::<BigEndian>(sign)?;
        // DScale
        out.write_u16::<BigEndian>(scale)?;
        // Now process the number
        for chunk in all_groups {
            let calculated = chunk.parse::<u16>().unwrap();
            out.write_u16::<BigEndian>(calculated.to_u16().unwrap())?;
        }
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            NUMERIC => true,
            _ => false,
        }
    }

    to_sql_checked!();
}

#[cfg(test)]
mod test {
    use super::*;
    use pg_crate::{Connection, TlsMode};
    use std::str::FromStr;

    fn read_type(sql_type: &str, checks: &[&'static str]) {
        let conn = match Connection::connect("postgres://postgres@localhost", TlsMode::None) {

            Ok(x) => x,
            Err(err) => panic!("{:#?}", err),
        };
        for &val in checks.iter() {
            let stmt = match conn.prepare(&*format!("SELECT {}::{}", val, sql_type)) {
                Ok(x) => x,
                Err(err) => panic!("{:#?}", err),
            };
            let result: Decimal = match stmt.query(&[]) {
                Ok(x) => x.iter().next().unwrap().get(0),
                Err(err) => panic!("{:#?}", err),
            };
            assert_eq!(val, result.to_string());
        }
    }

    fn write_type(sql_type: &str, checks: &[&'static str]) {
        let conn = match Connection::connect("postgres://postgres@localhost", TlsMode::None) {
            Ok(x) => x,
            Err(err) => panic!("{:#?}", err),
        };
        for &val in checks.iter() {
            let stmt = match conn.prepare(&*format!("SELECT $1::{}", sql_type)) {
                Ok(x) => x,
                Err(err) => panic!("{:#?}", err),
            };
            let number = Decimal::from_str(val).unwrap();
            let result: Decimal = match stmt.query(&[&number]) {
                Ok(x) => x.iter().next().unwrap().get(0),
                Err(err) => panic!("{:#?}", err),
            };
            assert_eq!(val, result.to_string());
        }
    }

    #[test]
    fn test_null() {
        let conn = match Connection::connect("postgres://postgres@localhost", TlsMode::None) {
            Ok(x) => x,
            Err(err) => panic!("{:#?}", err),
        };

        // Test NULL
        let stmt = match conn.prepare(&"SELECT NULL::numeric") {
            Ok(x) => x,
            Err(err) => panic!("{:#?}", err),
        };
        let result: Option<Decimal> = match stmt.query(&[]) {
            Ok(x) => x.iter().next().unwrap().get(0),
            Err(err) => panic!("{:#?}", err),
        };
        assert_eq!(None, result);
    }

    #[test]
    fn read_numeric_type() {
        read_type(
            "NUMERIC(26,6)",
            &[
                "3950.123456",
                "3950",
                "0.1",
                "0.01",
                "0.001",
                "0.0001",
                "0.00001",
                "0.000001",
                "1",
                "-100",
                "-123.456",
                "119996.25",
                "1000000",
                "9999999.99999",
            ],
        );
    }


    #[test]
    fn write_numeric_type() {
        write_type(
            "NUMERIC(26,6)",
            &[
                "3950.123456",
                "3950",
                "0.1",
                "0.01",
                "0.001",
                "0.0001",
                "0.00001",
                "0.000001",
                "1",
                "-100",
                "-123.456",
                "119996.25",
                "1000000",
                "9999999.99999",
            ],
        );
    }
}
