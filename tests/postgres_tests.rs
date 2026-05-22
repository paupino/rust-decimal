#![cfg(any(feature = "db-postgres", feature = "db-tokio-postgres"))]

#[test]
fn postgres_to_from_sql() {
    use bytes::BytesMut;
    use core::str::FromStr;
    use postgres::types::{FromSql, Kind, ToSql, Type};
    use rust_decimal::Decimal;

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
        input.to_sql(&t, &mut bytes).unwrap();
        let output = Decimal::from_sql(&t, &bytes).unwrap();

        assert_eq!(input, output);
    }
}

#[test]
fn postgres_from_sql_special_numeric() {
    use postgres::types::{FromSql, Kind, Type};
    use rust_decimal::Decimal;

    // The numbers below are the big-endian equivalent of the NUMERIC_* masks for NAN, PINF, NINF
    let tests = &[
        ("NaN", &[0, 0, 0, 0, 192, 0, 0, 0]),
        ("Infinity", &[0, 0, 0, 0, 208, 0, 0, 0]),
        ("-Infinity", &[0, 0, 0, 0, 240, 0, 0, 0]),
    ];

    let t = Type::new("".into(), 0, Kind::Simple, "".into());

    for (name, bytes) in tests {
        let res = Decimal::from_sql(&t, *bytes);
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
