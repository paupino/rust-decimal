use core::str::FromStr;
use rust_decimal::{Decimal, RoundingStrategy};

#[test]
fn it_can_round_to_2dp() {
    let a = Decimal::from_str("6.12345").unwrap();
    let b = (Decimal::from_str("100").unwrap() * a).round() / Decimal::from_str("100").unwrap();
    assert_eq!("6.12", b.to_string());
}

#[test]
fn it_can_round_using_basic_midpoint_rules() {
    let tests = &[
        ("3.5", RoundingStrategy::MidpointAwayFromZero, "4"),
        ("2.8", RoundingStrategy::MidpointAwayFromZero, "3"),
        ("2.5", RoundingStrategy::MidpointAwayFromZero, "3"),
        ("2.1", RoundingStrategy::MidpointAwayFromZero, "2"),
        ("-2.1", RoundingStrategy::MidpointAwayFromZero, "-2"),
        ("-2.5", RoundingStrategy::MidpointAwayFromZero, "-3"),
        ("-2.8", RoundingStrategy::MidpointAwayFromZero, "-3"),
        ("-3.5", RoundingStrategy::MidpointAwayFromZero, "-4"),
        ("3.5", RoundingStrategy::MidpointNearestEven, "4"),
        ("2.8", RoundingStrategy::MidpointNearestEven, "3"),
        ("2.5", RoundingStrategy::MidpointNearestEven, "2"),
        ("2.1", RoundingStrategy::MidpointNearestEven, "2"),
        ("-2.1", RoundingStrategy::MidpointNearestEven, "-2"),
        ("-2.5", RoundingStrategy::MidpointNearestEven, "-2"),
        ("-2.8", RoundingStrategy::MidpointNearestEven, "-3"),
        ("-3.5", RoundingStrategy::MidpointNearestEven, "-4"),
        ("3.5", RoundingStrategy::MidpointTowardZero, "3"),
        ("2.8", RoundingStrategy::MidpointTowardZero, "3"),
        ("2.5", RoundingStrategy::MidpointTowardZero, "2"),
        ("2.1", RoundingStrategy::MidpointTowardZero, "2"),
        ("-2.1", RoundingStrategy::MidpointTowardZero, "-2"),
        ("-2.5", RoundingStrategy::MidpointTowardZero, "-2"),
        ("-2.8", RoundingStrategy::MidpointTowardZero, "-3"),
        ("-3.5", RoundingStrategy::MidpointTowardZero, "-3"),
        ("2.8", RoundingStrategy::ToNegativeInfinity, "2"),
        ("2.5", RoundingStrategy::ToNegativeInfinity, "2"),
        ("2.1", RoundingStrategy::ToNegativeInfinity, "2"),
        ("-2.1", RoundingStrategy::ToNegativeInfinity, "-3"),
        ("-2.5", RoundingStrategy::ToNegativeInfinity, "-3"),
        ("-2.8", RoundingStrategy::ToNegativeInfinity, "-3"),
        ("2.8", RoundingStrategy::ToPositiveInfinity, "3"),
        ("2.5", RoundingStrategy::ToPositiveInfinity, "3"),
        ("2.1", RoundingStrategy::ToPositiveInfinity, "3"),
        ("-2.1", RoundingStrategy::ToPositiveInfinity, "-2"),
        ("-2.5", RoundingStrategy::ToPositiveInfinity, "-2"),
        ("-2.8", RoundingStrategy::ToPositiveInfinity, "-2"),
        ("2.8", RoundingStrategy::ToZero, "2"),
        ("2.5", RoundingStrategy::ToZero, "2"),
        ("2.1", RoundingStrategy::ToZero, "2"),
        ("-2.1", RoundingStrategy::ToZero, "-2"),
        ("-2.5", RoundingStrategy::ToZero, "-2"),
        ("-2.8", RoundingStrategy::ToZero, "-2"),
        ("2.8", RoundingStrategy::AwayFromZero, "3"),
        ("2.5", RoundingStrategy::AwayFromZero, "3"),
        ("2.1", RoundingStrategy::AwayFromZero, "3"),
        ("-2.1", RoundingStrategy::AwayFromZero, "-3"),
        ("-2.5", RoundingStrategy::AwayFromZero, "-3"),
        ("-2.8", RoundingStrategy::AwayFromZero, "-3"),
    ];

    for &(input, strategy, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(0, strategy);
        assert_eq!(expected, b.to_string(), "{input} > {expected} for {strategy:?}");
    }
}

#[test]
fn it_can_round_using_bankers_rounding() {
    let tests = &[
        ("6.12345", 2, "6.12"),
        ("6.126", 2, "6.13"),
        ("-6.126", 2, "-6.13"),
        ("6.5", 0, "6"),
        ("7.5", 0, "8"),
        ("1.2250", 2, "1.22"),
        ("1.2252", 2, "1.23"),
        ("1.2249", 2, "1.22"),
        ("6.1", 2, "6.1"),
        ("0.0000", 2, "0.00"),
        ("0.6666666666666666666666666666", 2, "0.67"),
        ("1.40", 0, "1"),
        ("2.60", 0, "3"),
        ("2.1234567890123456789012345678", 27, "2.123456789012345678901234568"),
    ];
    for &(input, dp, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven);
        assert_eq!(expected, b.to_string(), "MidpointNearestEven");
    }
}

#[test]
fn it_can_round_complex_numbers_using_bankers_rounding() {
    // Issue #71
    let rate = Decimal::new(19, 2); // 0.19
    let one = Decimal::new(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832
    let part = part.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven); // 0.16
    assert_eq!("0.16", part.to_string(), "MidpointNearestEven");
}

#[test]
fn it_can_round_using_round_half_up() {
    let tests = &[
        ("0", 0, "0"),
        ("1.234", 3, "1.234"),
        ("1.12", 5, "1.12"),
        ("6.34567", 2, "6.35"),
        ("6.5", 0, "7"),
        ("12.49", 0, "12"),
        ("0.6666666666666666666666666666", 2, "0.67"),
        ("1.40", 0, "1"),
        ("2.60", 0, "3"),
        ("2.1234567890123456789012345678", 27, "2.123456789012345678901234568"),
    ];
    for &(input, dp, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero);
        assert_eq!(expected, b.to_string(), "MidpointAwayFromZero");
    }
}

#[test]
fn it_can_round_complex_numbers_using_round_half_up() {
    // Issue #71
    let rate = Decimal::new(19, 2); // 0.19
    let one = Decimal::new(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832
    let part = part.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero); // 0.16
    assert_eq!("0.16", part.to_string(), "MidpointAwayFromZero");
}

#[test]
fn it_can_round_using_round_half_down() {
    let tests = &[
        ("0", 0, "0"),
        ("1.234", 3, "1.234"),
        ("1.12", 5, "1.12"),
        ("6.34567", 2, "6.35"),
        ("6.51", 0, "7"),
        ("12.5", 0, "12"),
        ("0.6666666666666666666666666666", 2, "0.67"),
        ("1.40", 0, "1"),
        ("2.60", 0, "3"),
        ("2.1234567890123456789012345678", 27, "2.123456789012345678901234568"),
    ];
    for &(input, dp, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(dp, RoundingStrategy::MidpointTowardZero);
        assert_eq!(expected, b.to_string(), "MidpointTowardZero");
    }
}

#[test]
fn it_can_round_complex_numbers_using_round_half_down() {
    // Issue #71
    let rate = Decimal::new(19, 2); // 0.19
    let one = Decimal::new(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832

    let part = part.round_dp_with_strategy(2, RoundingStrategy::MidpointTowardZero); // 0.16
    assert_eq!("0.16", part.to_string(), "RoundHalfDown");
}

#[test]
fn it_can_round_to_2dp_using_explicit_function() {
    let a = Decimal::from_str("6.12345").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("6.12", b.to_string());
}

#[test]
fn it_can_round_up_to_2dp_using_explicit_function() {
    let a = Decimal::from_str("6.126").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("6.13", b.to_string());
}

#[test]
fn it_can_round_down_to_2dp_using_explicit_function() {
    let a = Decimal::from_str("-6.126").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("-6.13", b.to_string());
}

#[test]
fn it_can_round_down_using_bankers_rounding() {
    let a = Decimal::from_str("6.5").unwrap();
    let b = a.round_dp(0u32);
    assert_eq!("6", b.to_string());
}

#[test]
fn it_can_round_up_using_bankers_rounding() {
    let a = Decimal::from_str("7.5").unwrap();
    let b = a.round_dp(0u32);
    assert_eq!("8", b.to_string());
}

#[test]
fn it_can_round_correctly_using_bankers_rounding_1() {
    let a = Decimal::from_str("1.2250").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("1.22", b.to_string());
}

#[test]
fn it_can_round_correctly_using_bankers_rounding_2() {
    let a = Decimal::from_str("1.2251").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("1.23", b.to_string());
}

#[test]
fn it_can_round_down_when_required() {
    let a = Decimal::from_str("1.2249").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("1.22", b.to_string());
}

#[test]
fn it_can_round_to_2dp_using_explicit_function_without_changing_value() {
    let a = Decimal::from_str("6.1").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("6.1", b.to_string());
}

#[test]
fn it_can_round_zero() {
    let a = Decimal::from_str("0.0000").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("0.00", b.to_string());
}

#[test]
fn it_can_round_large_decimals() {
    let a = Decimal::from_str("0.6666666666666666666666666666").unwrap();
    let b = a.round_dp(2u32);
    assert_eq!("0.67", b.to_string());
}

#[test]
fn it_can_round_simple_numbers_down() {
    let a = Decimal::from_str("1.40").unwrap();
    let b = a.round_dp(0u32);
    assert_eq!("1", b.to_string());
}

#[test]
fn it_can_round_simple_numbers_up() {
    let a = Decimal::from_str("2.60").unwrap();
    let b = a.round_dp(0u32);
    assert_eq!("3", b.to_string());
}

#[test]
fn it_can_round_simple_numbers_with_high_precision() {
    let a = Decimal::from_str("2.1234567890123456789012345678").unwrap();
    let b = a.round_dp(27u32);
    assert_eq!("2.123456789012345678901234568", b.to_string());
}

#[test]
fn it_can_round_complex_numbers() {
    // Issue #71
    let rate = Decimal::new(19, 2); // 0.19
    let one = Decimal::new(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832
    let part = part.round_dp(2); // 0.16
    assert_eq!("0.16", part.to_string());
}

#[test]
fn it_does_not_round_decimals_to_too_many_dp() {
    // Issue 574
    let zero = Decimal::new(0, 28);
    let rounded = zero.round_dp(32);
    assert_eq!(rounded.scale(), 28); // If dp > old_scale, we retain the old scale.
    rounded.to_string();
}

#[test]
fn it_can_round_down() {
    let tests = &[
        ("0.470", 1, "0.4"),
        ("-0.470", 1, "-0.4"), // Toward zero
        ("0.400", 1, "0.4"),
        ("-0.400", 1, "-0.4"),
    ];
    for &(input, dp, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(dp, RoundingStrategy::ToZero);
        assert_eq!(expected, b.to_string(), "ToZero");
    }
}

#[test]
fn it_can_round_up() {
    let tests = &[
        ("2.8", 0, "3"),
        ("2.5", 0, "3"),
        ("2.1", 0, "3"),
        ("-2.1", 0, "-3"),
        ("-2.5", 0, "-3"),
        ("-2.8", 0, "-3"),
        ("0.320", 1, "0.4"),
        ("-0.320", 1, "-0.4"),
        ("0.300", 1, "0.3"),
        ("-0.300", 1, "-0.3"),
    ];

    for &(input, dp, expected) in tests {
        let a = Decimal::from_str(input).unwrap();
        let b = a.round_dp_with_strategy(dp, RoundingStrategy::AwayFromZero);
        assert_eq!(expected, b.to_string(), "AwayFromZero");
    }
}

#[test]
fn it_can_round_significant_figures() {
    let tests = &[
        ("305.459", 0u32, Some("0")),
        ("305.459", 1, Some("300")),
        ("305.459", 2, Some("310")),
        ("305.459", 3, Some("305")),
        ("305.459", 4, Some("305.5")),
        ("305.459", 5, Some("305.46")),
        ("305.459", 6, Some("305.459")),
        ("305.459", 7, Some("305.4590")),
        ("305.459", 10, Some("305.4590000")),
        ("-305.459", 3, Some("-305")),
        ("-305.459", 2, Some("-310")), // We ignore the negative
        ("-305.459", 5, Some("-305.46")),
        (
            "79228162514264337593543950335",
            29,
            Some("79228162514264337593543950335"),
        ),
        ("79228162514264337593543950335", 1, None),
        (
            "79228162514264337593543950335",
            2,
            Some("79000000000000000000000000000"),
        ),
        (
            "79228162514264337593543950335",
            30,
            Some("79228162514264337593543950335"),
        ),
        (
            "79228162514264337593543950335",
            u32::MAX,
            Some("79228162514264337593543950335"),
        ),
    ];
    for &(input, sf, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let result = input.round_sf(sf);
        if let Some(expected) = expected {
            assert!(result.is_some(), "Expected result for {input}.round_sf({sf})");
            assert_eq!(expected, result.unwrap().to_string(), "{input}.round_sf({sf})");
        } else {
            assert!(result.is_none(), "Unexpected result for {input}.round_sf({sf})");
        }
    }
}

#[test]
fn it_can_round_significant_figures_with_strategy() {
    let tests = &[
        ("12301", 3u32, RoundingStrategy::AwayFromZero, Some("12400")),
        ("123.01", 3u32, RoundingStrategy::AwayFromZero, Some("124")),
        ("1.2301", 3u32, RoundingStrategy::AwayFromZero, Some("1.24")),
        ("0.12301", 3u32, RoundingStrategy::AwayFromZero, Some("0.124")),
        ("0.012301", 3u32, RoundingStrategy::AwayFromZero, Some("0.0124")),
        ("0.0000012301", 3u32, RoundingStrategy::AwayFromZero, Some("0.00000124")),
        ("1.012301", 3u32, RoundingStrategy::AwayFromZero, Some("1.02")),
    ];
    for &(input, sf, strategy, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let result = input.round_sf_with_strategy(sf, strategy);
        if let Some(expected) = expected {
            assert!(
                result.is_some(),
                "Expected result for {input}.round_sf_with_strategy({sf}, {strategy:?})"
            );
            assert_eq!(
                expected,
                result.unwrap().to_string(),
                "{input}.round_sf_with_strategy({sf}, {strategy:?})"
            );
        } else {
            assert!(
                result.is_none(),
                "Unexpected result for {input}.round_sf_with_strategy({sf}, {strategy:?})"
            );
        }
    }
}
