#[test]
fn postgres_zero_retains_scale() {
    use bytes::BytesMut;
    use postgres::types::{FromSql, ToSql, Type};
    use rust_decimal::Decimal;
    use rust_decimal_postgres::PgDecimal;

    for scale in 0..=Decimal::MAX_SCALE {
        for negative in [false, true] {
            let mut input = Decimal::from_parts(0, 0, 0, false, scale);
            input.set_sign_negative(negative);
            assert_eq!(input.is_sign_negative(), negative);
            let mut bytes = BytesMut::new();
            PgDecimal(input).to_sql(&Type::NUMERIC, &mut bytes).unwrap();

            // Zero has no digit groups and a positive sign, but retains its display scale.
            assert_eq!(&bytes[..], &[0, 0, 0, 0, 0, 0, 0, scale as u8]);
            let output = PgDecimal::from_sql(&Type::NUMERIC, &bytes).unwrap().0;
            assert!(output.is_zero());
            assert!(output.is_sign_positive());
            assert_eq!(output.scale(), scale);
        }
    }
}

#[test]
fn postgres_to_from_sql() {
    use bytes::BytesMut;
    use core::str::FromStr;
    use postgres::types::{FromSql, Kind, ToSql, Type};
    use rust_decimal::Decimal;
    use rust_decimal_postgres::PgDecimal;

    let tests = &[
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

    let t = Type::new("".into(), 0, Kind::Simple, "".into());

    for test in tests {
        let input = Decimal::from_str(test).unwrap();
        let mut bytes = BytesMut::new();
        PgDecimal(input).to_sql(&t, &mut bytes).unwrap();
        let output = PgDecimal::from_sql(&t, &bytes).unwrap().0;

        assert_eq!(input, output);
    }
}

#[test]
fn postgres_from_sql_special_numeric() {
    use postgres::types::{FromSql, Kind, Type};

    use rust_decimal_postgres::PgDecimal;

    // The numbers below are the big-endian equivalent of the NUMERIC_* masks for NAN, PINF, NINF
    let tests = &[
        ("NaN", &[0, 0, 0, 0, 192, 0, 0, 0]),
        ("Infinity", &[0, 0, 0, 0, 208, 0, 0, 0]),
        ("-Infinity", &[0, 0, 0, 0, 240, 0, 0, 0]),
    ];

    let t = Type::new("".into(), 0, Kind::Simple, "".into());

    for (name, bytes) in tests {
        let res = PgDecimal::from_sql(&t, *bytes);
        match &res {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains(name),
                    "Error message does not contain the expected value: {}",
                    name
                );
            }
        }
    }
}

#[test]
fn postgres_from_sql_exceeds_maximum_value() {
    use postgres::types::{FromSql, Kind, Type};
    use rust_decimal::Error;
    use rust_decimal_postgres::PgDecimal;

    // Single group of 1 with a large weight: 10000^weight. These fit into an i128 but exceed the
    // 96 bit mantissa of a Decimal, so they must return an error instead of panicking.
    let tests: &[(&str, &[u8])] = &[
        ("1e32", &[0x00, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
        ("1e36", &[0x00, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
    ];

    let t = Type::new("".into(), 0, Kind::Simple, "".into());

    for (name, bytes) in tests {
        match PgDecimal::from_sql(&t, bytes) {
            Ok(value) => panic!("Expected error for {}, got {}", name, value.0),
            Err(e) => assert_eq!(e.to_string(), Error::ExceedsMaximumPossibleValue.to_string()),
        }
    }
}

#[test]
fn postgres_from_sql_numeric_rounds_from_complete_wire_value() {
    use postgres::types::{FromSql, Kind, Type};

    use rust_decimal_postgres::PgDecimal;

    let mut raw = vec![0, 9, 0, 4, 0, 0, 0, 16];
    for group in [9999u16, 9999, 9999, 9999, 9999, 0, 0, 5000, 1] {
        raw.extend_from_slice(&group.to_be_bytes());
    }
    let ty = Type::new("".into(), 0, Kind::Simple, "".into());
    let actual = PgDecimal::from_sql(&ty, &raw).unwrap().0;
    assert_eq!(actual.mantissa(), 9999999999999999999900000001);
    assert_eq!(actual.scale(), 8);
}

#[test]
fn postgres_from_sql_numeric_rounds_negative_complete_wire_value() {
    use postgres::types::{FromSql, Kind, Type};

    use rust_decimal_postgres::PgDecimal;

    let mut raw = vec![0, 9, 0, 4, 0x40, 0, 0, 16];
    for group in [9999u16, 9999, 9999, 9999, 9999, 0, 0, 5000, 1] {
        raw.extend_from_slice(&group.to_be_bytes());
    }
    let ty = Type::new("".into(), 0, Kind::Simple, "".into());
    let actual = PgDecimal::from_sql(&ty, &raw).unwrap().0;
    assert_eq!(actual.mantissa(), -9999999999999999999900000001);
    assert_eq!(actual.scale(), 8);
}

#[test]
fn postgres_from_sql_rejects_invalid_digit_group() {
    use postgres::types::{FromSql, Kind, Type};

    use rust_decimal_postgres::PgDecimal;

    let raw = [0, 1, 0, 0, 0, 0, 0, 0, 0x27, 0x10];
    let ty = Type::new("".into(), 0, Kind::Simple, "".into());
    assert!(PgDecimal::from_sql(&ty, &raw).is_err());
}

#[test]
fn postgres_from_sql_numeric_rounding_controls() {
    use core::str::FromStr;
    use postgres::types::{FromSql, Kind, Type};
    use rust_decimal::Decimal;
    use rust_decimal_postgres::PgDecimal;

    let cases: &[(&[u16], i16, u16, bool, &str)] = &[
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 0, 4999, 9999],
            4,
            16,
            false,
            "99999999999999999999.00000000",
        ),
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 0, 4999, 9999],
            4,
            16,
            true,
            "-99999999999999999999.00000000",
        ),
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 0],
            4,
            16,
            false,
            "99999999999999999999.00000000",
        ),
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 0, 5000, 0],
            4,
            16,
            true,
            "-99999999999999999999.00000000",
        ),
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 1, 5000, 0],
            4,
            16,
            false,
            "99999999999999999999.00000002",
        ),
        (
            &[9999, 9999, 9999, 9999, 9999, 0, 1, 5000, 0],
            4,
            16,
            true,
            "-99999999999999999999.00000002",
        ),
        (&[5000], -8, 29, false, "0.0000000000000000000000000001"),
        (&[5000], -8, 29, true, "-0.0000000000000000000000000001"),
        (&[4999, 9000], -8, 33, false, "0.0000000000000000000000000000"),
        (&[5000, 1000], -8, 33, false, "0.0000000000000000000000000001"),
        (&[1, 2450, 9999, 9999, 9999, 9999, 9999, 9999], 0, 2, false, "1.25"),
    ];
    let ty = Type::new("".into(), 0, Kind::Simple, "".into());
    for (digits, weight, scale, negative, expected) in cases {
        let mut raw = Vec::new();
        raw.extend_from_slice(&(digits.len() as u16).to_be_bytes());
        raw.extend_from_slice(&weight.to_be_bytes());
        raw.extend_from_slice(&(if *negative { 0x4000u16 } else { 0 }).to_be_bytes());
        raw.extend_from_slice(&scale.to_be_bytes());
        for digit in *digits {
            raw.extend_from_slice(&digit.to_be_bytes());
        }
        let actual = PgDecimal::from_sql(&ty, &raw).unwrap().0;
        assert_eq!(actual, Decimal::from_str(expected).unwrap());
    }
}
