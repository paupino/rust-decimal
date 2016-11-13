extern crate byteorder;
extern crate postgres;
extern crate num;

use self::postgres::Result as PostgresResult;
use self::postgres::types::*;
use self::postgres::error::Error;

use std::io::prelude::*;
use std::fmt;
use std::result::*;
use std::error;

use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use self::num::{BigUint, One, Zero};
use self::num::bigint::ToBigUint;

#[cfg(test)]
use self::postgres::{Connection, SslMode};

use super::Decimal;
use std::str::FromStr;

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
    //  i16 dcsale. Number of digits (in base 10) to print after decimal separator
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
    //   result = result + 1234 * 0.001;
    // 15 E0 = 5600
    //   result = resNult + 5600 * 0.00000001;
    //

    fn from_sql<R: Read>(_: &Type, raw: &mut R, _: &SessionInfo) -> PostgresResult<Decimal> {
        let num_groups = try!(raw.read_u16::<BigEndian>());
        let weight = try!(raw.read_i16::<BigEndian>()); // 10000^weight
        // Sign: 0x0000 = positive, 0x4000 = negative, 0xC000 = NaN
        let sign = try!(raw.read_u16::<BigEndian>());
        // Number of digits (in base 10) to print after decimal separator
        let fixed_scale = try!(raw.read_u16::<BigEndian>()) as i32;

        // Build up a list of powers that will be used
        let mut powers = Vec::new();
        let mult = 10000.to_biguint().unwrap();
        let mut val: BigUint = One::one();
        powers.push(BigUint::one());
        for _ in 1..num_groups {
            val = val.clone() * mult.clone();
            powers.push(val.clone());
        }
        powers.reverse();

        // Now process the number
        let mut result: BigUint = Zero::zero();
        for i in 0..num_groups {
            let group = try!(raw.read_u16::<BigEndian>());
            let calculated = powers[i as usize].clone() * group.to_biguint().unwrap();
            // println!("{} {}", i, calculated);
            result = result + calculated;
        }

        // Finally, adjust for the scale
        // println!("num_groups: {}, weight {}, result {}", num_groups, weight, result);
        let mut scale = (num_groups as i16 - weight - 1) as i32 * 4;
        // Scale could be negative
        if scale < 0 {
            result = result * 10i64.pow((scale * -1) as u32).to_biguint().unwrap();
            scale = 0;
        } else if scale > fixed_scale {
            result = result / 10i64.pow((scale - fixed_scale) as u32).to_biguint().unwrap();
            scale = fixed_scale;
        }

        // Create the decimal
        let neg = sign == 0x4000;
        match Decimal::from_biguint(result, scale as u32, neg) {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::Conversion(Box::new(InvalidDecimal))),
        }
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::Numeric => true,
            _ => false,
        }
    }
}

#[cfg(test)]
fn test_type(sql_type: &str, checks: &[&'static str]) {
    let conn = match Connection::connect("postgres://paulmason@localhost", &SslMode::None) {
        Ok(x) => x,
        Err(err) => panic!("{:#?}", err),
    };
    for &val in checks.iter() {
        // println!("{}", val);
        let stmt = match conn.prepare(&*format!("SELECT {}::{}", val, sql_type)) {
            Ok(x) => x,
            Err(err) => panic!("{:#?}", err),
        };
        let result: Decimal = match stmt.query(&[]) {
            Ok(x) => x.iter().next().unwrap().get(0),
            Err(err) => panic!("{:#?}", err),
        };
        assert_eq!(val, result.to_string());
        //
        // let stmt = match conn.prepare(&*format!("SELECT $1::{}", sql_type)) {
        // Ok(x) => x,
        // Err(err) => panic!("{:#?}", err)
        // };
        // let number = Decimal::from_str(val).unwrap();
        // let result : Decimal = match stmt.query(&[&number]) {
        // Ok(x) => x.iter().next().unwrap().get(0),
        // Err(err) => panic!("{:#?}", err)
        // };
        // assert_eq!(val, result.to_string());
        //
    }

    // Also test NULL
    let stmt = match conn.prepare(&*format!("SELECT NULL::{}", sql_type)) {
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
fn it_can_read_numeric_type() {
    test_type("NUMERIC(26,6)",
              &["3950.123456", "3950", "0.000001", "-100", "-123.4560", "119996.2500", "1000000"]);
}
