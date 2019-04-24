use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use num::Zero;

use crate::Decimal;

use postgres::{to_sql_checked, types::*};

#[cfg(not(feature = "const_fn"))]
use lazy_static::lazy_static;

use std::{error, fmt, io::Cursor, result::*};

use crate::decimal::{div_by_u32, is_all_zero, mul_by_u32};

#[cfg(feature = "const_fn")]
const DECIMALS: [Decimal; 15] = [
    Decimal::from_parts(1, 0, 0, false, 28),
    Decimal::from_parts(1, 0, 0, false, 24),
    Decimal::from_parts(1, 0, 0, false, 20),
    Decimal::from_parts(1, 0, 0, false, 16),
    Decimal::from_parts(1, 0, 0, false, 12),
    Decimal::from_parts(1, 0, 0, false, 8),
    Decimal::from_parts(1, 0, 0, false, 4),
    Decimal::from_parts(1, 0, 0, false, 0),
    Decimal::from_parts(1_0000, 0, 0, false, 0),
    Decimal::from_parts(1_0000_0000, 0, 0, false, 0),
    Decimal::from_parts(
        1_0000_0000_0000u64 as u32,
        (1_0000_0000_0000u64 >> 32) as u32,
        0,
        false,
        0,
    ),
    Decimal::from_parts(
        1_0000_0000_0000_0000u64 as u32,
        (1_0000_0000_0000_0000u64 >> 32) as u32,
        0,
        false,
        0,
    ),
    Decimal::from_parts(1661992960, 1808227885, 5, false, 0),
    Decimal::from_parts(2701131776, 466537709, 54210, false, 0),
    Decimal::from_parts(268435456, 1042612833, 542101086, false, 0),
];

#[cfg(not(feature = "const_fn"))]
lazy_static! {

    // When procedural macro's are stabablized
    //  this will look MUCH better
    static ref DECIMALS: [Decimal; 15] = [
        Decimal::new(1, 28),
        Decimal::new(1, 24),
        Decimal::new(1, 20),
        Decimal::new(1, 16),
        Decimal::new(1, 12),
        Decimal::new(1, 8),
        Decimal::new(1, 4),
        Decimal::new(1, 0),
        Decimal::new(10000, 0),
        Decimal::new(100000000, 0),
        Decimal::new(1000000000000, 0),
        Decimal::new(10000000000000000, 0),
        Decimal::from_parts(1661992960, 1808227885, 5, false, 0),
        Decimal::from_parts(2701131776, 466537709, 54210, false, 0),
        Decimal::from_parts(268435456, 1042612833, 542101086, false, 0),
    ];
}

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

        // Read all of the groups
        let mut groups = Vec::new();
        for _ in 0..num_groups as usize {
            let group = raw.read_u16::<BigEndian>()?;
            groups.push(Decimal::new(group as i64, 0));
        }
        groups.reverse();

        // Now process the number
        let mut result = Decimal::zero();
        for (index, group) in groups.iter().enumerate() {
            result = result + (&DECIMALS[index + 7] * group);
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
        // If it's zero we can short cut with a u64
        if self.is_zero() {
            out.write_u64::<BigEndian>(0)?;
            return Ok(IsNull::No);
        }
        let sign = if self.is_sign_negative() { 0x4000 } else { 0x0000 };
        let scale = self.scale() as u16;

        let groups_diff = scale & 0x3; // groups_diff = scale % 4
        let mut fractional_groups_count = (scale >> 2) as isize; // fractional_groups_count = scale / 4
        fractional_groups_count += if groups_diff > 0 { 1 } else { 0 };

        let mut mantissa = self.mantissa_array4();

        if groups_diff > 0 {
            let remainder = 4 - groups_diff;
            let power = 10u32.pow(remainder as u32);
            mul_by_u32(&mut mantissa, power);
        }

        // array to store max mantissa of Decimal in Postgres decimal format
        const MAX_GROUP_COUNT: usize = 8;
        let mut groups = [0u16; MAX_GROUP_COUNT];

        let mut num_groups = 0usize;
        while !is_all_zero(&mantissa) {
            let group_digits = div_by_u32(&mut mantissa, 10000) as u16;
            groups[num_groups] = group_digits;
            num_groups += 1;
        }

        let whole_portion_len = num_groups as isize - fractional_groups_count;
        let weight = if whole_portion_len <= 0 {
            -(fractional_groups_count as i16)
        } else {
            whole_portion_len as i16 - 1
        };

        // Number of groups
        out.write_u16::<BigEndian>(num_groups as u16)?;
        // Weight of first group
        out.write_i16::<BigEndian>(weight)?;
        // Sign
        out.write_u16::<BigEndian>(sign)?;
        // DScale
        out.write_u16::<BigEndian>(scale)?;
        // Now process the number
        for group in groups[0..num_groups].iter().rev() {
            out.write_u16::<BigEndian>(*group)?;
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

    use postgres::{Connection, TlsMode};

    use std::str::FromStr;

    #[test]
    fn ensure_equivalent_decimal_constants() {
        let expected_decimals = [
            Decimal::new(1, 28),
            Decimal::new(1, 24),
            Decimal::new(1, 20),
            Decimal::new(1, 16),
            Decimal::new(1, 12),
            Decimal::new(1, 8),
            Decimal::new(1, 4),
            Decimal::new(1, 0),
            Decimal::new(10000, 0),
            Decimal::new(100000000, 0),
            Decimal::new(1000000000000, 0),
            Decimal::new(10000000000000000, 0),
            Decimal::from_parts(1661992960, 1808227885, 5, false, 0),
            Decimal::from_parts(2701131776, 466537709, 54210, false, 0),
            Decimal::from_parts(268435456, 1042612833, 542101086, false, 0),
        ];

        assert_eq!(&expected_decimals[..], &DECIMALS[..]);
    }

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

    pub static TEST_DECIMALS: &[&str; 20] = &[
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
        "12340.56789",
        "79228162514264337593543950335", // 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF (96 bit)
        "4951760157141521099596496895",  // 0x0FFF_FFFF_FFFF_FFFF_FFFF_FFFF (95 bit)
        "4951760157141521099596496896",  // 0x1000_0000_0000_0000_0000_0000
        "18446744073709551615",
        "-18446744073709551615",
    ];

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
        read_type("NUMERIC(35, 6)", TEST_DECIMALS);
    }

    #[test]
    fn write_numeric_type() {
        write_type("NUMERIC(35, 6)", TEST_DECIMALS);
    }
}
