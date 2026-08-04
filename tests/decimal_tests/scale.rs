use core::str::FromStr;
use rust_decimal::Decimal;

#[test]
fn it_can_trunc() {
    let tests = &[
        ("1.00000000000000000000", "1"),
        ("1.000000000000000000000001", "1"),
        ("1.123456789", "1"),
        ("1.9", "1"),
        ("1", "1"),
        // Also the inflection
        ("-1.00000000000000000000", "-1"),
        ("-1.000000000000000000000001", "-1"),
        ("-1.123456789", "-1"),
        ("-1.9", "-1"),
        ("-1", "-1"),
    ];

    for &(value, expected) in tests {
        let value = Decimal::from_str(value).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        let trunc = value.trunc();
        assert_eq!(expected.to_string(), trunc.to_string());
    }
}

#[test]
fn it_can_trunc_with_scale() {
    let cmp = Decimal::from_str("1.2345").unwrap();
    let tests = [
        "1.23450",
        "1.234500001",
        "1.23451",
        "1.23454",
        "1.23455",
        "1.23456",
        "1.23459",
        "1.234599999",
    ];
    for test in tests {
        assert_eq!(
            Decimal::from_str(test).unwrap().trunc_with_scale(4),
            cmp,
            "Original: {}",
            test
        );
    }

    let cmp = Decimal::from_str("-1.2345").unwrap();
    let tests = [
        "-1.23450",
        "-1.234500001",
        "-1.23451",
        "-1.23454",
        "-1.23455",
        "-1.23456",
        "-1.23459",
        "-1.234599999",
    ];
    for test in tests {
        assert_eq!(
            Decimal::from_str(test).unwrap().trunc_with_scale(4),
            cmp,
            "Original: {}",
            test
        );
    }

    // Complex cases
    let cmp = Decimal::from_str("0.5156").unwrap();
    let tests = [
        "0.51560089",
        "0.515600893",
        "0.5156008936",
        "0.51560089369",
        "0.515600893691",
        "0.5156008936910",
        "0.51560089369101",
        "0.515600893691016",
        "0.5156008936910161",
        "0.51560089369101613",
        "0.515600893691016134",
        "0.5156008936910161349",
        "0.51560089369101613494",
        "0.515600893691016134941",
        "0.5156008936910161349411",
        "0.51560089369101613494115",
        "0.515600893691016134941151",
        "0.5156008936910161349411515",
        "0.51560089369101613494115158",
        "0.515600893691016134941151581",
        "0.5156008936910161349411515818",
    ];
    for test in tests {
        assert_eq!(
            Decimal::from_str(test).unwrap().trunc_with_scale(4),
            cmp,
            "Original: {}",
            test
        );
    }
}

#[test]
fn it_can_fract() {
    let tests = &[
        ("1.00000000000000000000", "0.00000000000000000000"),
        ("1.000000000000000000000001", "0.000000000000000000000001"),
    ];

    for &(value, expected) in tests {
        let value = Decimal::from_str(value).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        let fract = value.fract();
        assert_eq!(expected.to_string(), fract.to_string());
    }
}

#[test]
fn it_can_normalize() {
    let tests = &[
        ("1.00000000000000000000", "1"),
        ("1.10000000000000000000000", "1.1"),
        ("1.00010000000000000000000", "1.0001"),
        ("1", "1"),
        ("1.1", "1.1"),
        ("1.0001", "1.0001"),
        ("-0", "0"),
        ("-0.0", "0"),
        ("-0.010", "-0.01"),
        ("0.0", "0"),
    ];

    for &(value, expected) in tests {
        let value = Decimal::from_str(value).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        let normalized = value.normalize();
        assert_eq!(expected.to_string(), normalized.to_string());
    }
}

#[test]
#[should_panic(expected = "Scale exceeds the maximum precision allowed: 29 > 28")]
fn it_panics_when_scale_too_large() {
    let _ = Decimal::from_i64_with_scale(1, 29);
}

#[test]
fn it_can_rescale() {
    let tests = &[
        ("0", 6, "0.000000", 6),
        ("0.000000", 2, "0.00", 2),
        ("0.12345600000", 6, "0.123456", 6),
        ("0.123456", 12, "0.123456000000", 12),
        ("0.123456", 0, "0", 0),
        ("0.000001", 4, "0.0000", 4),
        ("1233456", 4, "1233456.0000", 4),
        // Cap to 28
        ("1.2", 30, "1.2000000000000000000000000000", 28),
        ("79228162514264337593543950335", 0, "79228162514264337593543950335", 0),
        ("4951760157141521099596496895", 1, "4951760157141521099596496895.0", 1),
        ("4951760157141521099596496896", 1, "4951760157141521099596496896.0", 1),
        ("18446744073709551615", 6, "18446744073709551615.000000", 6),
        ("-18446744073709551615", 6, "-18446744073709551615.000000", 6),
        // 27 since we can't fit a scale of 28 for this number
        ("11.76470588235294", 28, "11.764705882352940000000000000", 27),
    ];

    for &(value_raw, new_scale, expected_value, expected_scale) in tests {
        let mut value = Decimal::from_str(value_raw).unwrap();
        value.rescale(new_scale);
        assert_eq!(expected_value, value.to_string());
        assert_eq!(expected_scale, value.scale());
    }
}

#[test]
fn it_creates_from_i64_with_scale() {
    let value = Decimal::from_i64_with_scale(1234, 2);
    assert_eq!("12.34", value.to_string());
    assert_eq!(2, value.scale());
}

#[test]
fn it_returns_error_for_excessive_scale() {
    let result = Decimal::try_from_i64_with_scale(1234, 29);
    assert!(result.is_err());
}

#[test]
fn it_creates_from_i64_with_scale_in_const_context() {
    const VALUE: Decimal = match Decimal::try_from_i64_with_scale(1234, 2) {
        Ok(d) => d,
        Err(_) => panic!("unexpected"),
    };
    assert_eq!("12.34", VALUE.to_string());
}

#[test]
#[allow(deprecated)]
fn deprecated_constructors_still_function() {
    assert_eq!(Decimal::new(1234, 2), Decimal::from_i64_with_scale(1234, 2));
    assert_eq!(
        Decimal::try_new(1234, 2).unwrap(),
        Decimal::try_from_i64_with_scale(1234, 2).unwrap()
    );
}
