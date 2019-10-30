use num::Zero;

use crate::Decimal;

use std::{convert::TryInto, error, fmt, result::*};

use crate::decimal::{div_by_u32, is_all_zero, mul_by_u32};

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

struct PostgresDecimal<D> {
    neg: bool,
    weight: i16,
    scale: u16,
    digits: D,
}

impl Decimal {
    fn from_postgres<D: ExactSizeIterator<Item = u16>>(
        PostgresDecimal {
            neg,
            weight,
            scale,
            digits,
        }: PostgresDecimal<D>,
    ) -> Result<Self, InvalidDecimal> {
        let num_groups = digits.len() as u16;
        // Number of digits (in base 10) to print after decimal separator
        let fixed_scale = scale as i32;

        // Read all of the groups
        let mut groups = digits
            .into_iter()
            .map(|d| Decimal::new(d as i64, 0))
            .collect::<Vec<_>>();
        groups.reverse();

        // Now process the number
        let mut result = Decimal::zero();
        for (index, group) in groups.iter().enumerate() {
            result = result + (DECIMALS[index + 7] * group);
        }

        // Finally, adjust for the scale
        let mut scale = (num_groups as i16 - weight - 1) as i32 * 4;
        // Scale could be negative
        if scale < 0 {
            result *= Decimal::new(10i64.pow((-scale) as u32), 0);
            scale = 0;
        } else if scale > fixed_scale {
            result /= Decimal::new(10i64.pow((scale - fixed_scale) as u32), 0);
            scale = fixed_scale;
        }

        // Create the decimal
        if result.set_scale(scale as u32).is_err() {
            return Err(InvalidDecimal);
        }
        result.set_sign(!neg);

        // Normalize by truncating any trailing 0's from the decimal representation
        Ok(result.normalize())
    }

    fn to_postgres(self) -> PostgresDecimal<Vec<i16>> {
        if self.is_zero() {
            return PostgresDecimal {
                neg: false,
                weight: 0,
                scale: 0,
                digits: vec![0],
            };
        }
        let scale = self.scale() as u16;

        let groups_diff = scale & 0x3; // groups_diff = scale % 4
        let mut fractional_groups_count = (scale >> 2) as isize; // fractional_groups_count = scale / 4
        fractional_groups_count += if groups_diff > 0 { 1 } else { 0 };

        let mut mantissa = self.mantissa_array4();

        if groups_diff > 0 {
            let remainder = 4 - groups_diff;
            let power = 10u32.pow(u32::from(remainder));
            mul_by_u32(&mut mantissa, power);
        }

        // array to store max mantissa of Decimal in Postgres decimal format
        const MAX_GROUP_COUNT: usize = 8;
        let mut digits = Vec::with_capacity(MAX_GROUP_COUNT);

        while !is_all_zero(&mantissa) {
            let digit = div_by_u32(&mut mantissa, 10000) as u16;
            digits.push(digit.try_into().unwrap());
        }
        digits.reverse();

        let whole_portion_len = digits.len() as isize - fractional_groups_count;
        let weight = if whole_portion_len < 0 {
            -(fractional_groups_count as i16)
        } else {
            whole_portion_len as i16 - 1
        };

        PostgresDecimal {
            neg: self.is_sign_negative(),
            digits,
            scale,
            weight,
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel {
    use super::*;

    use ::diesel::{
        deserialize::{self, FromSql},
        pg::data_types::PgNumeric,
        pg::Pg,
        serialize::{self, Output, ToSql},
        sql_types::Numeric,
    };
    use ::std::{
        convert::{TryFrom, TryInto},
        io::Write,
    };

    impl<'a> TryFrom<&'a PgNumeric> for Decimal {
        type Error = Box<dyn error::Error + Send + Sync>;

        fn try_from(numeric: &'a PgNumeric) -> deserialize::Result<Self> {
            let (neg, weight, scale, digits) = match *numeric {
                PgNumeric::Positive {
                    weight,
                    scale,
                    ref digits,
                } => (false, weight, scale, digits),
                PgNumeric::Negative {
                    weight,
                    scale,
                    ref digits,
                } => (true, weight, scale, digits),
                PgNumeric::NaN => return Err(Box::from("NaN is not supported in Decimal")),
            };

            Ok(Self::from_postgres(PostgresDecimal {
                neg,
                weight,
                scale,
                digits: digits.iter().copied().map(|v| v.try_into().unwrap()),
            })
            .map_err(Box::new)?)
        }
    }

    impl TryFrom<PgNumeric> for Decimal {
        type Error = Box<dyn error::Error + Send + Sync>;

        fn try_from(numeric: PgNumeric) -> deserialize::Result<Self> {
            (&numeric).try_into()
        }
    }

    impl<'a> From<&'a Decimal> for PgNumeric {
        // NOTE(clippy): Clippy suggests to replace the `.take_while(|i| i.is_zero())`
        // with `.take_while(Zero::is_zero)`, but that's a false positive.
        // The closure gets an `&&i16` due to autoderef `<i16 as Zero>::is_zero(&self) -> bool`
        // is called. There is no impl for `&i16` that would work with this closure.
        #[allow(clippy::assign_op_pattern, clippy::redundant_closure)]
        fn from(decimal: &'a Decimal) -> Self {
            let PostgresDecimal {
                neg,
                weight,
                scale,
                digits,
            } = decimal.to_postgres();

            let digits = digits.into_iter().map(|v| v.try_into().unwrap()).collect();

            if neg {
                PgNumeric::Negative { digits, scale, weight }
            } else {
                PgNumeric::Positive { digits, scale, weight }
            }
        }
    }

    impl From<Decimal> for PgNumeric {
        fn from(bigdecimal: Decimal) -> Self {
            (&bigdecimal).into()
        }
    }

    impl ToSql<Numeric, Pg> for Decimal {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
            let numeric = PgNumeric::from(self);
            ToSql::<Numeric, Pg>::to_sql(&numeric, out)
        }
    }

    impl FromSql<Numeric, Pg> for Decimal {
        fn from_sql(numeric: Option<&[u8]>) -> deserialize::Result<Self> {
            PgNumeric::from_sql(numeric)?.try_into()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::str::FromStr;

        #[test]
        fn decimal_to_pgnumeric_converts_digits_to_base_10000() {
            let decimal = Decimal::from_str("1").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 0,
                digits: vec![1],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("10").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 0,
                digits: vec![10],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("10000").unwrap();
            let expected = PgNumeric::Positive {
                weight: 1,
                scale: 0,
                digits: vec![1, 0],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("10001").unwrap();
            let expected = PgNumeric::Positive {
                weight: 1,
                scale: 0,
                digits: vec![1, 1],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("100000000").unwrap();
            let expected = PgNumeric::Positive {
                weight: 2,
                scale: 0,
                digits: vec![1, 0, 0],
            };
            assert_eq!(expected, decimal.into());
        }

        #[test]
        fn decimal_to_pg_numeric_properly_adjusts_scale() {
            let decimal = Decimal::from_str("1").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 0,
                digits: vec![1],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("1.0").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 1,
                digits: vec![1, 0],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("1.1").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 1,
                digits: vec![1, 1000],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("1.10").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 2,
                digits: vec![1, 1000],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("100000000.0001").unwrap();
            let expected = PgNumeric::Positive {
                weight: 2,
                scale: 4,
                digits: vec![1, 0, 0, 1],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("0.1").unwrap();
            let expected = PgNumeric::Positive {
                weight: -1,
                scale: 1,
                digits: vec![1000],
            };
            assert_eq!(expected, decimal.into());
        }

        #[test]
        #[cfg(feature = "unstable")]
        fn decimal_to_pg_numeric_retains_sign() {
            let decimal = Decimal::from_str("123.456").unwrap();
            let expected = PgNumeric::Positive {
                weight: 0,
                scale: 3,
                digits: vec![123, 4560],
            };
            assert_eq!(expected, decimal.into());

            let decimal = Decimal::from_str("-123.456").unwrap();
            let expected = PgNumeric::Negative {
                weight: 0,
                scale: 3,
                digits: vec![123, 4560],
            };
            assert_eq!(expected, decimal.into());
        }

        #[test]
        fn pg_numeric_to_decimal_works() {
            let expected = Decimal::from_str("50").unwrap();
            let pg_numeric = PgNumeric::Positive {
                weight: 0,
                scale: 0,
                digits: vec![50],
            };
            let res: Decimal = pg_numeric.try_into().unwrap();
            assert_eq!(res, expected);
            let expected = Decimal::from_str("123.456").unwrap();
            let pg_numeric = PgNumeric::Positive {
                weight: 0,
                scale: 3,
                digits: vec![123, 4560],
            };
            let res: Decimal = pg_numeric.try_into().unwrap();
            assert_eq!(res, expected);

            let expected = Decimal::from_str("-56.78").unwrap();
            let pg_numeric = PgNumeric::Negative {
                weight: 0,
                scale: 2,
                digits: vec![56, 7800],
            };
            let res: Decimal = pg_numeric.try_into().unwrap();
            assert_eq!(res, expected);
        }
    }
}

#[cfg(feature = "postgres")]
mod postgres {
    use super::*;

    use ::byteorder::{BigEndian, ReadBytesExt};
    use ::bytes::{BytesMut, BufMut};
    use ::tokio_postgres::types::*;
    use ::std::io::Cursor;

    impl<'a> FromSql<'a> for Decimal {
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

        fn from_sql(_: &Type, raw: &[u8]) -> Result<Decimal, Box<dyn error::Error + 'static + Sync + Send>> {
            let mut raw = Cursor::new(raw);
            let num_groups = raw.read_u16::<BigEndian>()?;
            let weight = raw.read_i16::<BigEndian>()?; // 10000^weight
                                                       // Sign: 0x0000 = positive, 0x4000 = negative, 0xC000 = NaN
            let sign = raw.read_u16::<BigEndian>()?;
            // Number of digits (in base 10) to print after decimal separator
            let scale = raw.read_u16::<BigEndian>()?;

            // Read all of the groups
            let mut groups = Vec::new();
            for _ in 0..num_groups as usize {
                groups.push(raw.read_u16::<BigEndian>()?);
            }

            Ok(Self::from_postgres(PostgresDecimal {
                neg: sign == 0x4000,
                weight,
                scale,
                digits: groups.into_iter(),
            })
            .map_err(Box::new)?)
        }

        fn accepts(ty: &Type) -> bool {
            ty.name() == "numeric"
        }
    }

    impl ToSql for Decimal {
        fn to_sql(&self, _: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn error::Error + 'static + Sync + Send>> {
            let PostgresDecimal {
                neg,
                weight,
                scale,
                digits,
            } = self.to_postgres();

            let num_digits = digits.len();

            out.reserve(8 + num_digits * 2);

            // Number of groups
            out.put_u16_be(num_digits.try_into().unwrap());
            // Weight of first group
            out.put_i16_be(weight);
            // Sign
            out.put_u16_be(if neg { 0x4000 } else { 0x0000 });
            // DScale
            out.put_u16_be(scale);
            // Now process the number
            for digit in digits[0..num_digits].iter() {
                out.put_i16_be(*digit);
            }

            Ok(IsNull::No)
        }

        fn accepts(ty: &Type) -> bool {
            ty.name() == "numeric"
        }

        to_sql_checked!();
    }

    #[cfg(test)]
    mod test {
        use super::*;

        use futures::future::FutureExt;
        use ::tokio_postgres::{connect, NoTls};

        use std::str::FromStr;

        pub static TEST_DECIMALS: &[(u32, u32, &str, &str)] = &[
            // precision, scale, sent, expected
            (35, 6, "3950.123456", "3950.123456"),
            (35, 2, "3950.123456", "3950.12"),
            (35, 2, "3950.1256", "3950.13"),
            (10, 2, "3950.123456", "3950.12"),
            (35, 6, "3950", "3950"),
            (4, 0, "3950", "3950"),
            (35, 6, "0.1", "0.1"),
            (35, 6, "0.01", "0.01"),
            (35, 6, "0.001", "0.001"),
            (35, 6, "0.0001", "0.0001"),
            (35, 6, "0.00001", "0.00001"),
            (35, 6, "0.000001", "0.000001"),
            (35, 6, "1", "1"),
            (35, 6, "-100", "-100"),
            (35, 6, "-123.456", "-123.456"),
            (35, 6, "119996.25", "119996.25"),
            (35, 6, "1000000", "1000000"),
            (35, 6, "9999999.99999", "9999999.99999"),
            (35, 6, "12340.56789", "12340.56789"),
            // 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF (96 bit)
            (35, 6, "79228162514264337593543950335", "79228162514264337593543950335"),
            // 0x0FFF_FFFF_FFFF_FFFF_FFFF_FFFF (95 bit)
            (35, 6, "4951760157141521099596496895", "4951760157141521099596496895"),
            // 0x1000_0000_0000_0000_0000_0000
            (35, 6, "4951760157141521099596496896", "4951760157141521099596496896"),
            (35, 6, "18446744073709551615", "18446744073709551615"),
            (35, 6, "-18446744073709551615", "-18446744073709551615"),
            (35, 6, "0.10001", "0.10001"),
            (35, 6, "0.12345", "0.12345"),
        ];

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

        #[tokio::test]
        async fn test_null() {
            let (client, connection) = connect("postgres://postgres@localhost", NoTls).await.unwrap();
            let connection = connection.map(|e| e.unwrap());
            tokio::spawn(connection);

            // Test NULL
            let statement = client.prepare(&"SELECT NULL::numeric").await.unwrap();
            let rows = client.query(&statement, &[]).await.unwrap();
            let result: Option<Decimal> = rows.iter().next().unwrap().get(0);

            assert_eq!(None, result);
        }

        #[tokio::test]
        async fn read_numeric_type() {
            let (client, connection) = connect("postgres://postgres@localhost", NoTls).await.unwrap();
            let connection = connection.map(|e| e.unwrap());
            tokio::spawn(connection);

            for &(precision, scale, sent, expected) in TEST_DECIMALS.iter() {
                let statement = client.prepare(&*format!("SELECT {}::NUMERIC({}, {})", sent, precision, scale)).await.unwrap();
                let rows = client.query(&statement, &[]).await.unwrap();
                let result: Decimal = rows.iter().next().unwrap().get(0);

                assert_eq!(expected, result.to_string(), "NUMERIC({}, {})", precision, scale);
            }
        }

        #[tokio::test]
        async fn write_numeric_type() {
            let (client, connection) = connect("postgres://postgres@localhost", NoTls).await.unwrap();
            let connection = connection.map(|e| e.unwrap());
            tokio::spawn(connection);

            for &(precision, scale, sent, expected) in TEST_DECIMALS.iter() {
                let statement = client.prepare(&*format!("SELECT $1::NUMERIC({}, {})", precision, scale)).await.unwrap();
                let number = Decimal::from_str(sent).unwrap();
                let rows = client.query(&statement, &[&number]).await.unwrap();
                let result: Decimal = rows.iter().next().unwrap().get(0);

                assert_eq!(expected, result.to_string(), "NUMERIC({}, {})", precision, scale);
            }
        }

        #[tokio::test]
        async fn numeric_overflow() {
            let tests = [(4, 4, "3950.1234")];
            let (client, connection) = connect("postgres://postgres@localhost", NoTls).await.unwrap();
            let connection = connection.map(|e| e.unwrap());
            tokio::spawn(connection);

            for &(precision, scale, sent) in tests.iter() {
                let statement = client.prepare(&*format!("SELECT {}::NUMERIC({}, {})", sent, precision, scale)).await.unwrap();

                match client.query(&statement, &[]).await {
                    Ok(_) => panic!(
                        "Expected numeric overflow for {}::NUMERIC({}, {})",
                        sent, precision, scale
                    ),
                    Err(err) => assert_eq!("22003", err.code().unwrap().code(), "Unexpected error code"),
                }
            }
        }
    }
}
