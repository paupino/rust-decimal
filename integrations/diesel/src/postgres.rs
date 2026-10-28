use crate::DieselDecimal;
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    pg::data_types::PgNumeric,
    serialize::{self, Output, ToSql},
    sql_types::Numeric,
};
use rust_decimal::{Decimal, Error};
use rust_decimal_pg_numeric::{PostgresDecimal, from_postgres, to_postgres};
use std::error;

impl<'a> TryFrom<&'a PgNumeric> for DieselDecimal {
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

        let digits: Vec<u16> = digits
            .iter()
            .map(|&v| -> deserialize::Result<u16> {
                if !(0..=9999).contains(&v) {
                    Err(Box::from(format!("invalid digit group: {v}")))
                } else {
                    Ok(v as u16)
                }
            })
            .collect::<deserialize::Result<_>>()?;
        let Some(result) = from_postgres(PostgresDecimal {
            neg,
            weight,
            scale,
            digits: digits.into_iter(),
        }) else {
            return Err(Box::new(Error::ExceedsMaximumPossibleValue));
        };
        Ok(Self(result))
    }
}

impl TryFrom<PgNumeric> for DieselDecimal {
    type Error = Box<dyn error::Error + Send + Sync>;

    fn try_from(numeric: PgNumeric) -> deserialize::Result<Self> {
        (&numeric).try_into()
    }
}

impl<'a> From<&'a DieselDecimal> for PgNumeric {
    fn from(decimal: &'a DieselDecimal) -> Self {
        let PostgresDecimal {
            neg,
            weight,
            scale,
            digits,
        } = to_postgres(decimal.0);

        if neg {
            PgNumeric::Negative { digits, scale, weight }
        } else {
            PgNumeric::Positive { digits, scale, weight }
        }
    }
}

impl From<DieselDecimal> for PgNumeric {
    fn from(decimal: DieselDecimal) -> Self {
        (&decimal).into()
    }
}

impl ToSql<Numeric, Pg> for DieselDecimal {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let numeric = PgNumeric::from(self);
        ToSql::<Numeric, Pg>::to_sql(&numeric, &mut out.reborrow())
    }
}

impl FromSql<Numeric, Pg> for DieselDecimal {
    fn from_sql(numeric: diesel::pg::PgValue) -> deserialize::Result<Self> {
        PgNumeric::from_sql(numeric)?.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn decimal_to_pg_numeric_zero_retains_scale() {
        for scale in 0..=Decimal::MAX_SCALE {
            for negative in [false, true] {
                let mut input = Decimal::from_parts(0, 0, 0, false, scale);
                input.set_sign_negative(negative);
                assert_eq!(input.is_sign_negative(), negative);
                let numeric = PgNumeric::from(DieselDecimal(input));
                assert_eq!(
                    numeric,
                    PgNumeric::Positive {
                        weight: 0,
                        scale: scale as u16,
                        digits: vec![],
                    }
                );
                let output = DieselDecimal::try_from(numeric).unwrap().0;
                assert!(output.is_zero());
                assert!(output.is_sign_positive());
                assert_eq!(output.scale(), scale);
            }
        }
    }

    #[test]
    fn negative_digit_group_returns_error_not_panic() {
        // Diesel stores PgNumeric digits as i16; a negative value must not panic via unwrap
        let pg = PgNumeric::Positive {
            weight: 0,
            scale: 0,
            digits: vec![-1i16],
        };
        let result = DieselDecimal::try_from(pg);
        assert!(result.is_err(), "expected Err for negative digit, got {result:?}");
    }

    #[test]
    fn test_unnecessary_zeroes() {
        fn extract(value: &str) -> Decimal {
            Decimal::from_str(value).unwrap()
        }

        let tests = &[
            ("0.000001660"),
            ("41.120255926293000"),
            ("0.5538973300"),
            ("08883.55986854293100"),
            ("0.0000_0000_0016_6000_00"),
            ("0.00000166650000"),
            ("1666500000000"),
            ("1666500000000.0000054500"),
            ("8944.000000000000"),
        ];

        for &value in tests {
            let value = extract(value);
            let pg = PgNumeric::from(DieselDecimal(value));
            let dec = DieselDecimal::try_from(pg).unwrap().0;
            assert_eq!(dec, value);
        }
    }

    #[test]
    fn decimal_to_pgnumeric_converts_digits_to_base_10000() {
        let decimal = Decimal::from_str("1").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 0,
            digits: vec![1],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("10").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 0,
            digits: vec![10],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("10000").unwrap();
        let expected = PgNumeric::Positive {
            weight: 1,
            scale: 0,
            digits: vec![1, 0],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("10001").unwrap();
        let expected = PgNumeric::Positive {
            weight: 1,
            scale: 0,
            digits: vec![1, 1],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("100000000").unwrap();
        let expected = PgNumeric::Positive {
            weight: 2,
            scale: 0,
            digits: vec![1, 0, 0],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());
    }

    #[test]
    fn decimal_to_pg_numeric_properly_adjusts_scale() {
        let decimal = Decimal::from_str("1").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 0,
            digits: vec![1],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("1.0").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 1,
            digits: vec![1],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("1.1").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 1,
            digits: vec![1, 1000],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("1.10").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 2,
            digits: vec![1, 1000],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("100000000.0001").unwrap();
        let expected = PgNumeric::Positive {
            weight: 2,
            scale: 4,
            digits: vec![1, 0, 0, 1],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("0.1").unwrap();
        let expected = PgNumeric::Positive {
            weight: -1,
            scale: 1,
            digits: vec![1000],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());
    }

    #[test]
    fn decimal_to_pg_numeric_retains_sign() {
        let decimal = Decimal::from_str("123.456").unwrap();
        let expected = PgNumeric::Positive {
            weight: 0,
            scale: 3,
            digits: vec![123, 4560],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());

        let decimal = Decimal::from_str("-123.456").unwrap();
        let expected = PgNumeric::Negative {
            weight: 0,
            scale: 3,
            digits: vec![123, 4560],
        };
        assert_eq!(expected, DieselDecimal(decimal).into());
    }

    #[test]
    fn pg_numeric_to_decimal_works() {
        let expected = Decimal::from_str("50").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 0,
            digits: vec![50],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res, expected);
        let expected = Decimal::from_str("123.456").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 3,
            digits: vec![123, 4560],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res, expected);

        let expected = Decimal::from_str("-56.78").unwrap();
        let pg_numeric = PgNumeric::Negative {
            weight: 0,
            scale: 2,
            digits: vec![56, 7800],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res, expected);

        // Verify no trailing zeroes are lost.

        let expected = Decimal::from_str("1.100").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 3,
            digits: vec![1, 1000],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());

        // To represent 5.00, Postgres can return either [5, 0] as the list of digits.
        let expected = Decimal::from_str("5.00").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 2,

            digits: vec![5, 0],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());

        // To represent 5.00, Postgres can return [5] as the list of digits.
        let expected = Decimal::from_str("5.00").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 2,
            digits: vec![5],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());

        let expected = Decimal::from_str("3.1415926535897932384626433833").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 30,
            digits: vec![3, 1415, 9265, 3589, 7932, 3846, 2643, 3832, 7950, 2800],
        };
        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());

        let expected = Decimal::from_str("3.1415926535897932384626433833").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 34,
            digits: vec![3, 1415, 9265, 3589, 7932, 3846, 2643, 3832, 7950, 2800],
        };

        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());

        let expected = Decimal::from_str("1.2345678901234567890123456790").unwrap();
        let pg_numeric = PgNumeric::Positive {
            weight: 0,
            scale: 34,
            digits: vec![1, 2345, 6789, 123, 4567, 8901, 2345, 6789, 5000, 0],
        };

        let res = DieselDecimal::try_from(pg_numeric).unwrap().0;
        assert_eq!(res.to_string(), expected.to_string());
    }

    #[test]
    fn pg_numeric_to_decimal_rounds_from_complete_value() {
        let numeric = PgNumeric::Positive {
            weight: 4,
            scale: 16,
            digits: vec![9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 1],
        };
        let actual = DieselDecimal::try_from(numeric).unwrap().0;
        assert_eq!(actual.mantissa(), 9999999999999999999900000001);
        assert_eq!(actual.scale(), 8);
    }

    #[test]
    fn pg_numeric_to_decimal_rounds_negative_complete_value() {
        let numeric = PgNumeric::Negative {
            weight: 4,
            scale: 16,
            digits: vec![9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 1],
        };
        let actual = DieselDecimal::try_from(numeric).unwrap().0;
        assert_eq!(actual.mantissa(), -9999999999999999999900000001);
        assert_eq!(actual.scale(), 8);
    }

    #[test]
    fn pg_numeric_to_decimal_rounding_controls() {
        let cases = [
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 0, 4999, 9999],
                4,
                16,
                false,
                "99999999999999999999.00000000",
            ),
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 0, 4999, 9999],
                4,
                16,
                true,
                "-99999999999999999999.00000000",
            ),
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 0],
                4,
                16,
                false,
                "99999999999999999999.00000000",
            ),
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 0],
                4,
                16,
                true,
                "-99999999999999999999.00000000",
            ),
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 1, 5000, 0],
                4,
                16,
                false,
                "99999999999999999999.00000002",
            ),
            (
                vec![9999, 9999, 9999, 9999, 9999, 0, 1, 5000, 0],
                4,
                16,
                true,
                "-99999999999999999999.00000002",
            ),
            (vec![5000], -8, 29, false, "0.0000000000000000000000000001"),
            (vec![5000], -8, 29, true, "-0.0000000000000000000000000001"),
            (vec![4999, 9000], -8, 33, false, "0.0000000000000000000000000000"),
            (vec![5000, 1000], -8, 33, false, "0.0000000000000000000000000001"),
            (vec![1, 2450, 9999, 9999, 9999, 9999, 9999, 9999], 0, 2, false, "1.25"),
        ];

        for (digits, weight, scale, negative, expected) in cases {
            let numeric = if negative {
                PgNumeric::Negative { weight, scale, digits }
            } else {
                PgNumeric::Positive { weight, scale, digits }
            };
            let actual = DieselDecimal::try_from(numeric).unwrap().0;
            assert_eq!(actual.to_string(), expected);
        }
    }
}
