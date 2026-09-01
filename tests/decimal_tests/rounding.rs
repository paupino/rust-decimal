use core::str::FromStr;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;

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
    let rate = Decimal::from_i64_with_scale(19, 2); // 0.19
    let one = Decimal::from_i64_with_scale(1, 0); // 1
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
    let rate = Decimal::from_i64_with_scale(19, 2); // 0.19
    let one = Decimal::from_i64_with_scale(1, 0); // 1
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
    let rate = Decimal::from_i64_with_scale(19, 2); // 0.19
    let one = Decimal::from_i64_with_scale(1, 0); // 1
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
    let rate = Decimal::from_i64_with_scale(19, 2); // 0.19
    let one = Decimal::from_i64_with_scale(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832
    let part = part.round_dp(2); // 0.16
    assert_eq!("0.16", part.to_string());
}

#[test]
fn it_does_not_round_decimals_to_too_many_dp() {
    // Issue 574
    let zero = Decimal::from_i64_with_scale(0, 28);
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

#[test]
fn round_sf_large_digits_does_not_overflow() {
    // scale + digits overflows u32 in debug builds when digits = u32::MAX
    assert!(dec!(1.5).round_sf(u32::MAX).is_some());
    assert!(dec!(-1.5).round_sf(u32::MAX).is_some());
    assert!(Decimal::MAX.round_sf(u32::MAX).is_some());
}

#[test]
fn round_sf_does_not_add_figures_on_carry() {
    // A round up that carries into a new leading digit (e.g. 0.95 -> 1.0) must not keep the
    // trailing zero: it is significant here, giving one more figure than requested.
    let tests = &[
        ("0.95", 1u32, "1"),
        ("0.99", 1, "1"),
        ("0.995", 2, "1.0"),
        ("0.099", 1, "0.1"),
        ("0.0095", 1, "0.01"),
        ("0.0099", 1, "0.01"),
        ("9.95", 2, "10"),
        ("99.95", 3, "100"),
        ("9999999.95", 8, "10000000"),
        ("99999999999999999999.95", 21, "100000000000000000000"),
        ("0.000000000000000000000000095", 1, "0.0000000000000000000000001"),
        ("-0.95", 1, "-1"),
        ("-0.995", 2, "-1.0"),
        ("-9.95", 2, "-10"),
    ];
    for &(input, sf, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let result = input.round_sf(sf).unwrap();
        assert_eq!(expected, result.to_string(), "{input}.round_sf({sf})");
        // an already rounded value must round to itself
        let again = result.round_sf(sf).unwrap();
        assert_eq!(
            result.serialize(),
            again.serialize(),
            "{input}.round_sf({sf}) is not idempotent"
        );
    }
}

#[test]
fn round_sf_carry_respects_rounding_strategy() {
    let tests = &[
        ("0.95", 1u32, RoundingStrategy::MidpointAwayFromZero, "1"),
        ("0.95", 1, RoundingStrategy::MidpointTowardZero, "0.9"),
        ("0.96", 1, RoundingStrategy::MidpointAwayFromZero, "1"),
        ("0.91", 1, RoundingStrategy::AwayFromZero, "1"),
        ("0.0949", 1, RoundingStrategy::AwayFromZero, "0.1"),
        ("0.994", 2, RoundingStrategy::AwayFromZero, "1.0"),
        ("0.99", 1, RoundingStrategy::ToPositiveInfinity, "1"),
        ("-0.99", 1, RoundingStrategy::ToNegativeInfinity, "-1"),
        ("-0.99", 1, RoundingStrategy::ToZero, "-0.9"),
    ];
    for &(input, sf, strategy, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let result = input.round_sf_with_strategy(sf, strategy).unwrap();
        assert_eq!(
            expected,
            result.to_string(),
            "{input}.round_sf_with_strategy({sf}, {strategy:?})"
        );
    }
}

#[test]
fn round_sf_keeps_legitimate_trailing_zeros() {
    let tests = &[
        // the carry stops before adding a digit: the trailing zero is significant
        ("2.95", 2u32, "3.0"),
        ("1.00", 2, "1.0"),
        ("10.0", 3, "10.0"),
        ("9.95", 3, "9.95"),
        // integral result: the scale cannot go negative
        ("99.5", 2, "100"),
        ("0.045", 1, "0.04"),
        ("0.012301", 3, "0.0123"),
        ("0", 5, "0"),
    ];
    for &(input, sf, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let result = input.round_sf(sf).unwrap();
        assert_eq!(expected, result.to_string(), "{input}.round_sf({sf})");
    }
}

// Figures shown by `to_string`: for integral values trailing zeros are placeholders, not
// significant.
fn displayed_figures(value: &Decimal) -> u32 {
    let digits = value.mantissa().unsigned_abs().to_string();
    let digits = digits.trim_start_matches('0');
    let digits = if value.scale() == 0 {
        digits.trim_end_matches('0')
    } else {
        digits
    };
    digits.len() as u32
}

#[test]
fn round_sf_carry_sweep_keeps_requested_figures() {
    let strategies = &[
        RoundingStrategy::MidpointNearestEven,
        RoundingStrategy::MidpointAwayFromZero,
        RoundingStrategy::MidpointTowardZero,
        RoundingStrategy::ToZero,
        RoundingStrategy::AwayFromZero,
        RoundingStrategy::ToNegativeInfinity,
        RoundingStrategy::ToPositiveInfinity,
    ];
    // deterministic xorshift, biased towards all-nines runs where rounding carries
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for i in 0..2000 {
        let power = 10i128.pow((next() % 26) as u32 + 1);
        let mantissa = match i % 3 {
            0 => power - (next() % 100) as i128,
            1 => power + (next() % 100) as i128,
            _ => (next() as i128) % power + 1,
        };
        let mantissa = if i % 2 == 0 { mantissa } else { -mantissa };
        let value = Decimal::from_i128_with_scale(mantissa, (next() % 29) as u32);
        for &strategy in strategies {
            for sf in 1..=4u32 {
                let Some(result) = value.round_sf_with_strategy(sf, strategy) else {
                    continue;
                };
                assert!(
                    displayed_figures(&result) <= sf,
                    "{value}.round_sf_with_strategy({sf}, {strategy:?}) = {result} has too many figures"
                );
                let again = result.round_sf_with_strategy(sf, strategy).unwrap();
                assert_eq!(
                    result.serialize(),
                    again.serialize(),
                    "{value}.round_sf_with_strategy({sf}, {strategy:?}) is not idempotent"
                );
            }
        }
    }
}

#[test]
fn round_sf_up_scale_stays_within_max_scale() {
    // Requesting more figures than the magnitude holds up-scales the value by padding zeros. The
    // target scale must be capped at MAX_SCALE: an uncapped target produced a malformed scale
    // 29-31 Decimal (e.g. 5e-27.round_sf(5) -> scale 31) that then panics on `to_string`.
    let strategies = &[
        RoundingStrategy::MidpointNearestEven,
        RoundingStrategy::MidpointAwayFromZero,
        RoundingStrategy::MidpointTowardZero,
        RoundingStrategy::ToZero,
        RoundingStrategy::AwayFromZero,
        RoundingStrategy::ToNegativeInfinity,
        RoundingStrategy::ToPositiveInfinity,
    ];
    // (input, digits, expected) - up-scaling only pads zeros, so every strategy agrees.
    let tests = &[
        // over the boundary: without the cap these were scale 29/30/31
        ("0.000000000000000000000000005", 5u32, "0.0000000000000000000000000050"),
        ("0.000000000000000000000000005", 4, "0.0000000000000000000000000050"),
        ("0.000000000000000000000000005", 3, "0.0000000000000000000000000050"),
        ("-0.000000000000000000000000005", 5, "-0.0000000000000000000000000050"),
        ("0.05", 28, "0.0500000000000000000000000000"),
        ("0.005", 27, "0.0050000000000000000000000000"),
        ("0.5", 29, "0.5000000000000000000000000000"),
        ("-0.5", 30, "-0.5000000000000000000000000000"),
        // in range: the cap does not interfere with normal up-scaling
        ("0.5", 3, "0.500"),
        ("305.459", 7, "305.4590"),
    ];
    for &(input, digits, expected) in tests {
        let value = Decimal::from_str(input).unwrap();
        for &strategy in strategies {
            let result = value
                .round_sf_with_strategy(digits, strategy)
                .expect("up-scaling a representable value must not return None");
            assert!(
                result.scale() <= Decimal::MAX_SCALE,
                "{input}.round_sf_with_strategy({digits}, {strategy:?}) scale {} exceeds MAX_SCALE",
                result.scale()
            );
            // `to_string` panics on a malformed scale, so this also guards the crash.
            assert_eq!(
                expected,
                result.to_string(),
                "{input}.round_sf_with_strategy({digits}, {strategy:?})"
            );
            // an already up-scaled value must round to itself
            let again = result.round_sf_with_strategy(digits, strategy).unwrap();
            assert_eq!(
                result.serialize(),
                again.serialize(),
                "{input}.round_sf_with_strategy({digits}, {strategy:?}) is not idempotent"
            );
        }
    }
}
