use core::str::FromStr;
use num_traits::ToPrimitive;
use rust_decimal::{Decimal, Error};

#[test]
fn it_can_parse_from_i32() {
    use num_traits::FromPrimitive;

    let tests = &[
        (0i32, "0"),
        (1i32, "1"),
        (-1i32, "-1"),
        (i32::MAX, "2147483647"),
        (i32::MIN, "-2147483648"),
    ];
    for &(input, expected) in tests {
        let parsed = Decimal::from_i32(input).unwrap();
        assert_eq!(
            expected,
            parsed.to_string(),
            "expected {expected} does not match parsed {parsed}"
        );
        assert_eq!(
            input.to_string(),
            parsed.to_string(),
            "i32 to_string {input} does not match parsed {parsed}"
        );
    }
}

#[test]
fn it_can_parse_from_i64() {
    use num_traits::FromPrimitive;

    let tests = &[
        (0i64, "0"),
        (1i64, "1"),
        (-1i64, "-1"),
        (i64::MAX, "9223372036854775807"),
        (i64::MIN, "-9223372036854775808"),
    ];
    for &(input, expected) in tests {
        let parsed = Decimal::from_i64(input).unwrap();
        assert_eq!(
            expected,
            parsed.to_string(),
            "expected {expected} does not match parsed {parsed}"
        );
        assert_eq!(
            input.to_string(),
            parsed.to_string(),
            "i64 to_string {input} does not match parsed {parsed}"
        );
    }
}

#[test]
fn it_can_go_from_and_into() {
    let d = Decimal::from_str("5").unwrap();
    let di8: Decimal = 5u8.into();
    let di32: Decimal = 5i32.into();
    let disize: Decimal = 5isize.into();
    let di64: Decimal = 5i64.into();
    let du8: Decimal = 5u8.into();
    let du32: Decimal = 5u32.into();
    let dusize: Decimal = 5usize.into();
    let du64: Decimal = 5u64.into();

    assert_eq!(d, di8);
    assert_eq!(di8, di32);
    assert_eq!(di32, disize);
    assert_eq!(disize, di64);
    assert_eq!(di64, du8);
    assert_eq!(du8, du32);
    assert_eq!(du32, dusize);
    assert_eq!(dusize, du64);
}

#[test]
fn it_converts_to_f64() {
    let tests = &[
        ("5", Some(5f64)),
        ("-5", Some(-5f64)),
        ("0.1", Some(0.1f64)),
        ("0.0", Some(0f64)),
        ("-0.0", Some(0f64)),
        ("0.0000000000025", Some(0.25e-11f64)),
        ("1000000.0000000000025", Some(1e6f64)),
        ("0.000000000000000000000000025", Some(0.25e-25_f64)),
        (
            "2.1234567890123456789012345678",
            Some(2.1234567890123456789012345678_f64),
        ),
        ("21234567890123456789012345678", Some(21234567890123458000000000000_f64)),
        (
            "-21234567890123456789012345678",
            Some(-21234567890123458000000000000_f64),
        ),
        ("1.59283191", Some(1.59283191_f64)),
        ("2.2238", Some(2.2238_f64)),
        ("2.2238123", Some(2.2238123_f64)),
        ("22238", Some(22238_f64)),
        ("1000000", Some(1000000_f64)),
        ("1000000.000000000000000000", Some(1000000_f64)),
        ("10000", Some(10000_f64)),
        ("10000.000000000000000000", Some(10000_f64)),
        ("100000", Some(100000_f64)),
        ("100000.000000000000000000", Some(100000_f64)),
    ];
    for &(value, expected) in tests {
        let value = Decimal::from_str(value).unwrap().to_f64();
        assert_eq!(expected, value);
    }
}

#[test]
fn it_converts_to_f64_nearest() {
    // Values with 16 or more significant digits. The reference is the standard library's parser,
    // which returns the nearest f64.
    let tests = [
        "42374010.16301119",
        "367360219.0653660",
        "41564725111192.09",
        "41178314804.09036",
        "7266536452.7374987",
        "964550194732601.1",
        "9533848929109301.5",
    ];
    for value in tests {
        let expected: f64 = value.parse().unwrap();
        assert_eq!(Decimal::from_str(value).unwrap().to_f64(), Some(expected), "{value}");
        assert_eq!(
            Decimal::from_str(&format!("-{value}")).unwrap().to_f64(),
            Some(-expected),
            "-{value}"
        );
    }
}

#[test]
fn it_converts_to_f64_at_the_edges() {
    let tests = [
        // Either side of the single-division domain: mantissa 2^53 - 1 and 2^53 at scale 22, and
        // 2^53 - 1 at scale 23.
        ("0.0000009007199254740991", 9.007199254740991e-7),
        ("0.0000009007199254740992", 9.007199254740992e-7),
        ("0.00000009007199254740991", 9.007199254740992e-8),
        // Ties resolve to the even significand, in both directions.
        ("9007199254740993", 9007199254740992.0),
        ("4503599627370496.5", 4503599627370496.0),
        ("4503599627370497.5", 4503599627370498.0),
        // A hair above a tie rounds up.
        ("4503599627370496.5000000000001", 4503599627370497.0),
        // The quotient's dropped bits are exactly half, so only the remainder of the division
        // decides: above the tie rounds up, the tie itself goes to the even significand.
        ("0.0000000000000000000000001338", 1.338e-25),
        ("34359738368.000003814697265626", 34359738368.00001),
        ("34359738368.000003814697265625", 34359738368.0),
        // Rounding up carries into the next power of two.
        ("0.99999999999999999", 1.0),
        // The ends of the range.
        ("79228162514264337593543950335", 7.922816251426434e28),
        ("7.9228162514264337593543950335", 7.9228162514264335),
        ("0.0000000000000000000000000001", 1e-28),
    ];
    for (value, expected) in tests {
        assert_eq!(value.parse::<f64>().unwrap(), expected, "{value}: expectation");
        assert_eq!(Decimal::from_str(value).unwrap().to_f64(), Some(expected), "{value}");
    }
}

#[test]
fn it_converts_negative_zero_to_negative_zero() {
    for value in [-Decimal::ZERO, Decimal::from_str("-0.4").unwrap().trunc()] {
        assert_eq!(value.to_string(), "-0");
        assert_eq!(value.to_f64().map(f64::to_bits), Some((-0.0f64).to_bits()), "{value:?}");
    }
}

#[test]
fn it_converts_to_f64_like_str_parse() {
    // Compare against the standard library's parser over mantissas of every bit width and every
    // scale. A fixed-seed generator keeps the test deterministic without a dependency.
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for width in 1..=96u32 {
        for scale in 0..=28u32 {
            for _ in 0..8 {
                let random = (u128::from(next()) << 64) | u128::from(next());
                let mantissa = (random & ((1u128 << width) - 1)) | (1u128 << (width - 1));
                let lo = mantissa as u32;
                let mid = (mantissa >> 32) as u32;
                let hi = (mantissa >> 64) as u32;
                for negative in [false, true] {
                    let value = Decimal::from_parts(lo, mid, hi, negative, scale);
                    let text = value.to_string();
                    let expected: f64 = text.parse().unwrap();
                    assert_eq!(value.to_f64(), Some(expected), "{text}");
                }
            }
        }
    }
}

#[test]
fn it_converts_to_f64_try() {
    let tests = &[
        ("5", Some(5f64)),
        ("-5", Some(-5f64)),
        ("0.1", Some(0.1f64)),
        ("0.0", Some(0f64)),
        ("-0.0", Some(0f64)),
        ("0.0000000000025", Some(0.25e-11f64)),
        ("1000000.0000000000025", Some(1e6f64)),
        ("0.000000000000000000000000025", Some(0.25e-25_f64)),
        (
            "2.1234567890123456789012345678",
            Some(2.1234567890123456789012345678_f64),
        ),
        ("21234567890123456789012345678", Some(21234567890123458000000000000_f64)),
        (
            "-21234567890123456789012345678",
            Some(-21234567890123458000000000000_f64),
        ),
        ("1.59283191", Some(1.59283191_f64)),
    ];
    for &(value, expected) in tests {
        let value = Decimal::from_str(value).unwrap().try_into().ok();
        assert_eq!(expected, value);
    }
}

#[test]
fn it_converts_to_i64() {
    let tests = [
        ("5", Some(5_i64)),
        ("-5", Some(-5_i64)),
        ("5.12345", Some(5_i64)),
        ("-5.12345", Some(-5_i64)),
        ("-9223372036854775808", Some(-9223372036854775808_i64)),
        ("-9223372036854775808", Some(i64::MIN)),
        ("9223372036854775807", Some(9223372036854775807_i64)),
        ("9223372036854775807", Some(i64::MAX)),
        ("-9223372036854775809", None), // i64::MIN - 1
        ("9223372036854775808", None),  // i64::MAX + 1
        // Clear overflows in hi bit
        ("-92233720368547758089", None),
        ("92233720368547758088", None),
    ];
    for (input, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        let actual = input.to_i64();
        assert_eq!(expected, actual, "Input: {input}");
    }
}

#[test]
fn it_converts_to_u64() {
    assert_eq!(5u64, Decimal::from_str("5").unwrap().to_u64().unwrap());
    assert_eq!(None, Decimal::from_str("-5").unwrap().to_u64());
    assert_eq!(5u64, Decimal::from_str("5.12345").unwrap().to_u64().unwrap());
    assert_eq!(
        0xFFFF_FFFF_FFFF_FFFF,
        Decimal::from_str("18446744073709551615").unwrap().to_u64().unwrap()
    );
    assert_eq!(None, Decimal::from_str("18446744073709551616").unwrap().to_u64());
}

#[test]
fn it_converts_to_i128() {
    let tests = &[
        ("5", Some(5i128)),
        ("-5", Some(-5i128)),
        ("5.12345", Some(5i128)),
        ("-5.12345", Some(-5i128)),
        ("9223372036854775807", Some(0x7FFF_FFFF_FFFF_FFFF)),
        ("92233720368547758089", Some(92233720368547758089i128)),
    ];
    for (dec, expected) in tests {
        assert_eq!(Decimal::from_str(dec).unwrap().to_i128(), *expected);
    }

    assert_eq!(
        79_228_162_514_264_337_593_543_950_335_i128,
        Decimal::MAX.to_i128().unwrap()
    );
}

#[test]
fn it_converts_to_u128() {
    let tests = &[
        ("5", Some(5u128)),
        ("-5", None),
        ("5.12345", Some(5u128)),
        ("-5.12345", None),
        ("18446744073709551615", Some(0xFFFF_FFFF_FFFF_FFFF)),
        ("18446744073709551616", Some(18446744073709551616u128)),
    ];
    for (dec, expected) in tests {
        assert_eq!(Decimal::from_str(dec).unwrap().to_u128(), *expected);
    }
    assert_eq!(
        79_228_162_514_264_337_593_543_950_335_u128,
        Decimal::MAX.to_u128().unwrap()
    );
}

#[test]
fn it_converts_from_i128() {
    let tests: &[(i128, Option<&str>)] = &[
        (5, Some("5")),
        (-5, Some("-5")),
        (0x7FFF_FFFF_FFFF_FFFF, Some("9223372036854775807")),
        (92233720368547758089, Some("92233720368547758089")),
        (0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF, Some("79228162514264337593543950335")),
        (0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF, None),
        (i128::MIN, None),
        (i128::MAX, None),
    ];
    for (value, expected) in tests {
        let from_i128 = num_traits::FromPrimitive::from_i128(*value);

        match expected {
            Some(expected_value) => {
                let decimal = Decimal::from_str(expected_value).unwrap();
                assert_eq!(from_i128, Some(decimal));
            }
            None => assert!(from_i128.is_none()),
        }
    }
}

#[test]
fn it_converts_from_u128() {
    let tests: &[(u128, Option<&str>)] = &[
        (5, Some("5")),
        (0xFFFF_FFFF_FFFF_FFFF, Some("18446744073709551615")),
        (0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF, Some("79228162514264337593543950335")),
        (0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF, None),
        (u128::MAX, None),
    ];
    for (value, expected) in tests {
        let from_u128 = num_traits::FromPrimitive::from_u128(*value);

        match expected {
            Some(expected_value) => {
                let decimal = Decimal::from_str(expected_value).unwrap();
                assert_eq!(from_u128, Some(decimal));
            }
            None => assert!(from_u128.is_none()),
        }
    }
}

#[test]
fn it_converts_from_str() {
    assert_eq!(Decimal::try_from("1").unwrap(), Decimal::ONE);
    assert_eq!(Decimal::try_from("10").unwrap(), Decimal::TEN);
}

#[test]
fn it_converts_from_f32() {
    use num_traits::FromPrimitive;

    let tests = [
        (0.1_f32, "0.1"),
        (1_f32, "1"),
        (0_f32, "0"),
        (0.12345_f32, "0.12345"),
        (0.1234567800123456789012345678_f32, "0.12345678"),
        (0.12345678901234567890123456789_f32, "0.12345679"),
        (0.00000000000000000000000000001_f32, "0"),
        (5.1_f32, "5.1"),
    ];

    for &(input, expected) in &tests {
        assert_eq!(
            expected,
            Decimal::from_f32(input).unwrap().to_string(),
            "from_f32({input})"
        );
        assert_eq!(
            expected,
            Decimal::try_from(input).unwrap().to_string(),
            "try_from({input})"
        );
    }
}

#[test]
fn it_converts_from_f32_limits() {
    use num_traits::FromPrimitive;

    assert!(Decimal::from_f32(f32::NAN).is_none(), "from_f32(f32::NAN)");
    assert!(Decimal::from_f32(f32::INFINITY).is_none(), "from_f32(f32::INFINITY)");
    assert!(Decimal::try_from(f32::NAN).is_err(), "try_from(f32::NAN)");
    assert!(Decimal::try_from(f32::INFINITY).is_err(), "try_from(f32::INFINITY)");

    // These overflow
    assert!(Decimal::from_f32(f32::MAX).is_none(), "from_f32(f32::MAX)");
    assert!(Decimal::from_f32(f32::MIN).is_none(), "from_f32(f32::MIN)");
    assert!(Decimal::try_from(f32::MAX).is_err(), "try_from(f32::MAX)");
    assert!(Decimal::try_from(f32::MIN).is_err(), "try_from(f32::MIN)");
}

#[test]
fn it_converts_from_f32_retaining_bits() {
    let tests = [
        (0.1_f32, "0.100000001490116119384765625"),
        (2_f32, "2"),
        (4.000_f32, "4"),
        (5.1_f32, "5.099999904632568359375"),
    ];

    for &(input, expected) in &tests {
        assert_eq!(
            expected,
            Decimal::from_f32_retain(input).unwrap().to_string(),
            "from_f32_retain({input})"
        );
    }
}

#[test]
fn it_converts_from_f64() {
    use num_traits::FromPrimitive;

    let tests = [
        (0.1_f64, "0.1"),
        (1_f64, "1"),
        (0_f64, "0"),
        (0.12345_f64, "0.12345"),
        (0.1234567890123456089012345678_f64, "0.1234567890123456"),
        (0.12345678901234567890123456789_f64, "0.1234567890123457"),
        (0.00000000000000000000000000001_f64, "0"),
        (0.6927_f64, "0.6927"),
        (0.00006927_f64, "0.00006927"),
        (0.000000006927_f64, "0.000000006927"),
        (5.1_f64, "5.1"),
    ];

    for &(input, expected) in &tests {
        assert_eq!(
            expected,
            Decimal::from_f64(input).unwrap().to_string(),
            "from_f64({input})"
        );
        assert_eq!(
            expected,
            Decimal::try_from(input).unwrap().to_string(),
            "try_from({input})"
        );
    }
}

#[test]
fn it_converts_from_f64_limits() {
    use num_traits::FromPrimitive;

    assert!(Decimal::from_f64(f64::NAN).is_none(), "from_f64(f64::NAN)");
    assert!(Decimal::from_f64(f64::INFINITY).is_none(), "from_f64(f64::INFINITY)");
    assert!(Decimal::try_from(f64::NAN).is_err(), "try_from(f64::NAN)");
    assert!(Decimal::try_from(f64::INFINITY).is_err(), "try_from(f64::INFINITY)");

    // These overflow
    assert!(Decimal::from_f64(f64::MAX).is_none(), "from_f64(f64::MAX)");
    assert!(Decimal::from_f64(f64::MIN).is_none(), "from_f64(f64::MIN)");
    assert!(Decimal::try_from(f64::MAX).is_err(), "try_from(f64::MIN)");
    assert!(Decimal::try_from(f64::MIN).is_err(), "try_from(f64::MAX)");
}

#[test]
fn it_converts_from_f64_dec_limits() {
    use num_traits::FromPrimitive;

    // Note Decimal MAX is: 79_228_162_514_264_337_593_543_950_335
    let over_max = 79_228_162_514_264_355_185_729_994_752_f64;
    let max_plus_one = 79_228_162_514_264_337_593_543_950_336_f64;
    let under_max = 79_228_162_514_264_328_797_450_928_128_f64;

    assert!(
        Decimal::from_f64(over_max).is_none(),
        "from_f64(79_228_162_514_264_355_185_729_994_752_f64) -> none (too large)"
    );
    assert!(
        Decimal::from_f64(max_plus_one).is_none(),
        "from_f64(79_228_162_514_264_337_593_543_950_336_f64) -> none (too large)"
    );
    assert_eq!(
        "79228162514264328797450928128",
        Decimal::from_f64(under_max).unwrap().to_string(),
        "from_f64(79_228_162_514_264_328_797_450_928_128_f64) -> some (inside limits)"
    );
}

#[test]
fn it_converts_from_f64_retaining_bits() {
    let tests = [
        (0.1_f64, "0.1000000000000000055511151231"),
        (2_f64, "2"),
        (4.000_f64, "4"),
        (5.1_f64, "5.0999999999999996447286321175"),
    ];

    for &(input, expected) in &tests {
        assert_eq!(
            expected,
            Decimal::from_f64_retain(input).unwrap().to_string(),
            "from_f64_retain({input})"
        );
    }
}

#[test]
fn it_converts_to_integers() {
    assert_eq!(i64::try_from(Decimal::ONE), Ok(1));
    assert_eq!(i64::try_from(Decimal::MAX), Err(Error::ConversionTo("i64")));
    assert_eq!(u128::try_from(Decimal::ONE_HUNDRED), Ok(100));
}

#[test]
fn it_handles_simple_underflow() {
    // Issue #71
    let rate = Decimal::new(19, 2); // 0.19
    let one = Decimal::new(1, 0); // 1
    let part = rate / (rate + one); // 0.19 / (0.19 + 1) = 0.1596638655462184873949579832
    let result = one * part;
    assert_eq!("0.1596638655462184873949579832", result.to_string());

    // 169 * 0.1596638655462184873949579832 = 26.983193277310924
    let result = part * Decimal::new(169, 0);
    assert_eq!("26.983193277310924369747899161", result.to_string());
    let result = Decimal::new(169, 0) * part;
    assert_eq!("26.983193277310924369747899161", result.to_string());
}

#[test]
#[should_panic(expected = "Number less than minimum value that can be represented.")]
fn it_handles_i128_min() {
    let _ = Decimal::from_i128_with_scale(i128::MIN, 0);
}

#[test]
fn it_handles_i128_min_safely() {
    let result = Decimal::try_from_i128_with_scale(i128::MIN, 0);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), Error::LessThanMinimumPossibleValue);
}
