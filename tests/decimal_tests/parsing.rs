use core::str::FromStr;
use rust_decimal::{Decimal, Error};

#[test]
fn it_creates_a_new_negative_decimal() {
    let a = Decimal::from_i64_with_scale(-100, 2);
    assert!(a.is_sign_negative());
    assert_eq!(a.scale(), 2);
    assert_eq!("-1.00", a.to_string());
}

#[test]
fn it_creates_a_new_decimal_using_numeric_boundaries() {
    let a = Decimal::from_i64_with_scale(i64::MAX, 2);
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 2);
    assert_eq!("92233720368547758.07", a.to_string());

    let b = Decimal::from_i64_with_scale(i64::MIN, 2);
    assert!(b.is_sign_negative());
    assert_eq!(b.scale(), 2);
    assert_eq!("-92233720368547758.08", b.to_string());
}

#[test]
fn it_parses_empty_string() {
    assert!(Decimal::from_str("").is_err());
    assert!(Decimal::from_str(" ").is_err());
}

#[test]
fn it_parses_positive_int_string() {
    let a = Decimal::from_str("233").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 0);
    assert_eq!("233", a.to_string());
}

#[test]
fn it_parses_negative_int_string() {
    let a = Decimal::from_str("-233").unwrap();
    assert!(a.is_sign_negative());
    assert_eq!(a.scale(), 0);
    assert_eq!("-233", a.to_string());
}

#[test]
fn it_parses_positive_float_string() {
    let a = Decimal::from_str("233.323223").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 6);
    assert_eq!("233.323223", a.to_string());
}

#[test]
fn it_parses_negative_float_string() {
    let a = Decimal::from_str("-233.43343").unwrap();
    assert!(a.is_sign_negative());
    assert_eq!(a.scale(), 5);
    assert_eq!("-233.43343", a.to_string());
}

#[test]
fn it_parses_positive_tiny_float_string() {
    let a = Decimal::from_str(".000001").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 6);
    assert_eq!("0.000001", a.to_string());
}

#[test]
fn it_parses_negative_tiny_float_string() {
    let a = Decimal::from_str("-0.000001").unwrap();
    assert!(a.is_sign_negative());
    assert_eq!(a.scale(), 6);
    assert_eq!("-0.000001", a.to_string());
}

#[test]
fn it_parses_big_integer_string() {
    let a = Decimal::from_str("79228162514264337593543950330").unwrap();
    assert_eq!("79228162514264337593543950330", a.to_string());
}

#[test]
fn it_parses_big_float_string() {
    let a = Decimal::from_str("79.228162514264337593543950330").unwrap();
    assert_eq!("79.228162514264337593543950330", a.to_string());
}

#[test]
fn it_parses_scientific_notation_from_str() {
    let a = Decimal::from_str("1.23e4").unwrap();
    assert_eq!("12300", a.to_string());

    let b = Decimal::from_str("6.7e-1").unwrap();
    assert_eq!("0.67", b.to_string());

    let c = Decimal::from_str("1E2").unwrap();
    assert_eq!("100", c.to_string());

    let d = Decimal::from_str("-2.5E-3").unwrap();
    assert_eq!("-0.0025", d.to_string());

    let e = Decimal::from_str("5e0").unwrap();
    assert_eq!("5", e.to_string());
}

#[test]
fn it_can_serialize_deserialize() {
    let tests = [
        "12.3456789",
        "5233.9008808150288439427720175",
        "-5233.9008808150288439427720175",
    ];
    for test in &tests {
        let a = Decimal::from_str(test).unwrap();
        let bytes = a.serialize();
        let b = Decimal::deserialize(bytes);
        assert_eq!(test.to_string(), b.to_string());
    }
}

#[test]
fn it_can_deserialize_unbounded_values() {
    // Mantissa for these: 19393111376951473493673267553
    let tests = [
        (
            [1u8, 0, 28, 206, 97, 81, 216, 182, 20, 30, 165, 78, 18, 155, 169, 62],
            // Scale 28: -1.9393111376951473493673267553
            "-1.9393111376951473493673267553",
        ),
        (
            [1u8, 0, 29, 206, 97, 81, 216, 182, 20, 30, 165, 78, 18, 155, 169, 62],
            // Scale 29: -0.19393111376951473493673267553
            "-0.1939311137695147349367326755",
        ),
        (
            [1u8, 0, 30, 206, 97, 81, 216, 182, 20, 30, 165, 78, 18, 155, 169, 62],
            // Scale 30: -0.019393111376951473493673267553
            "-0.0193931113769514734936732676",
        ),
        (
            [1u8, 0, 31, 206, 97, 81, 216, 182, 20, 30, 165, 78, 18, 155, 169, 62],
            // Scale 31: -0.0019393111376951473493673267553
            "-0.0019393111376951473493673268",
        ),
    ];
    for &(bytes, expected) in &tests {
        let dec = Decimal::deserialize(bytes);
        let string = format!("{dec:.9999}");
        let dec2 = Decimal::from_str(&string).unwrap();
        assert_eq!(dec, dec2);
        assert_eq!(dec.to_string(), expected, "dec.to_string()");
        assert_eq!(dec2.to_string(), expected, "dec2.to_string()");
    }
}

#[test]
fn it_can_parse_highly_significant_numbers() {
    let tests = &[
        ("11.111111111111111111111111111", "11.111111111111111111111111111"),
        ("11.11111111111111111111111111111", "11.111111111111111111111111111"),
        ("11.1111111111111111111111111115", "11.111111111111111111111111112"),
        ("115.111111111111111111111111111", "115.11111111111111111111111111"),
        ("1115.11111111111111111111111111", "1115.1111111111111111111111111"),
        ("11.1111111111111111111111111195", "11.111111111111111111111111120"),
        ("99.9999999999999999999999999995", "100.00000000000000000000000000"),
        ("-11.1111111111111111111111111195", "-11.111111111111111111111111120"),
        ("-99.9999999999999999999999999995", "-100.00000000000000000000000000"),
        ("3.1415926535897932384626433832", "3.1415926535897932384626433832"),
        (
            "8808257419827262908.5944405087133154018",
            "8808257419827262908.594440509",
        ),
        (
            "8097370036018690744.2590371109596744091",
            "8097370036018690744.259037111",
        ),
        (
            "8097370036018690744.2590371149596744091",
            "8097370036018690744.259037115",
        ),
        (
            "8097370036018690744.2590371159596744091",
            "8097370036018690744.259037116",
        ),
        ("1.234567890123456789012345678949999", "1.2345678901234567890123456789"),
        (".00000000000000000000000000001", "0.0000000000000000000000000000"),
        (".10000000000000000000000000000", "0.1000000000000000000000000000"),
    ];
    for &(value, expected) in tests {
        assert_eq!(expected, Decimal::from_str(value).unwrap().to_string());
    }
}

#[test]
fn it_can_parse_exact_highly_significant_numbers() {
    use rust_decimal::Error;

    let tests = &[
        (
            "11.111111111111111111111111111",
            Ok("11.111111111111111111111111111".to_string()),
        ),
        ("11.11111111111111111111111111111", Err(Error::Underflow)),
        ("11.1111111111111111111111111115", Err(Error::Underflow)),
        ("115.111111111111111111111111111", Err(Error::Underflow)),
        ("1115.11111111111111111111111111", Err(Error::Underflow)),
        ("11.1111111111111111111111111195", Err(Error::Underflow)),
        ("99.9999999999999999999999999995", Err(Error::Underflow)),
        ("-11.1111111111111111111111111195", Err(Error::Underflow)),
        ("-99.9999999999999999999999999995", Err(Error::Underflow)),
        (
            "3.1415926535897932384626433832",
            Ok("3.1415926535897932384626433832".to_string()),
        ),
        ("8808257419827262908.5944405087133154018", Err(Error::Underflow)),
        ("8097370036018690744.2590371109596744091", Err(Error::Underflow)),
        ("8097370036018690744.2590371149596744091", Err(Error::Underflow)),
        ("8097370036018690744.2590371159596744091", Err(Error::Underflow)),
        ("1.234567890123456789012345678949999", Err(Error::Underflow)),
        (".00000000000000000000000000001", Err(Error::Underflow)),
        (".10000000000000000000000000000", Err(Error::Underflow)),
    ];
    for &(value, ref expected) in tests.iter() {
        let actual = Decimal::from_str_exact(value).map(|d| d.to_string());
        assert_eq!(*expected, actual);
    }
}

#[test]
fn it_can_parse_alternative_formats() {
    let tests = &[
        ("1_000", "1000"),
        ("1_000_000", "1000000"),
        ("10_000_000", "10000000"),
        ("100_000", "100000"),
        // At the moment, we'll accept this
        ("1_____________0", "10"),
    ];
    for &(value, expected) in tests {
        assert_eq!(expected, Decimal::from_str(value).unwrap().to_string());
    }
}

#[test]
fn it_can_parse_fractional_numbers_with_underscore_separators() {
    let a = Decimal::from_str("0.1_23_456").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 6);
    assert_eq!("0.123456", a.to_string());
}

#[test]
fn it_can_parse_numbers_with_underscore_separators_before_decimal_point() {
    let a = Decimal::from_str("1_234.56").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 2);
    assert_eq!("1234.56", a.to_string());
}

#[test]
fn it_can_parse_numbers_and_round_correctly_with_underscore_separators_before_decimal_point() {
    let tests = &[
        (
            "8_097_370_036_018_690_744.2590371159596744091",
            "8097370036018690744.259037116",
        ),
        (
            "8097370036018690744.259_037_115_959_674_409_1",
            "8097370036018690744.259037116",
        ),
        (
            "8_097_370_036_018_690_744.259_037_115_959_674_409_1",
            "8097370036018690744.259037116",
        ),
    ];
    for &(value, expected) in tests {
        assert_eq!(expected, Decimal::from_str(value).unwrap().to_string());
    }
}

#[test]
fn it_can_reject_invalid_formats() {
    let tests = &["_1", "1.0.0", "10_00.0_00.0"];
    for &value in tests {
        assert!(
            Decimal::from_str(value).is_err(),
            "This succeeded unexpectedly: {value}"
        );
    }
}

#[test]
fn it_can_reject_large_numbers_with_panic() {
    let tests = &[
        // The maximum number supported is 79,228,162,514,264,337,593,543,950,335
        "79228162514264337593543950336",
        "79228162514264337593543950337",
        "79228162514264337593543950338",
        "79228162514264337593543950339",
        "79228162514264337593543950340",
    ];
    for &value in tests {
        if let Ok(out) = Decimal::from_str(value) {
            panic!("Unexpectedly parsed {value} into {out}")
        }
    }
}

#[test]
fn it_can_parse_individual_parts() {
    let pi = Decimal::from_parts(1102470952, 185874565, 1703060790, false, 28);
    assert_eq!(pi.to_string(), "3.1415926535897932384626433832");
}

#[test]
fn it_can_parse_scientific_notation_exact() {
    let tests = &[
        ("9.7e-7", Ok("0.00000097".to_string())),
        ("9e-7", Ok("0.0000009".to_string())),
        ("1.2e10", Ok("12000000000".to_string())),
        ("1.2e+10", Ok("12000000000".to_string())),
        ("12e10", Ok("120000000000".to_string())),
        ("9.7E-7", Ok("0.00000097".to_string())),
        ("1.2345E-24", Ok("0.0000000000000000000000012345".to_string())),
        ("12345E-28", Ok("0.0000000000000000000000012345".to_string())),
        ("1.2345E0", Ok("1.2345".to_string())),
        ("1E28", Ok("10000000000000000000000000000".to_string())),
        (
            "-20165.4676_e-+4294967292",
            Err(Error::ScaleExceedsMaximumPrecision(4294967292)),
        ),
    ];

    for &(value, ref expected) in tests {
        let actual = Decimal::from_scientific_exact(value).map(|d| d.to_string());
        assert_eq!(*expected, actual);
    }
}

#[test]
fn it_errors_parsing_large_scientific_notation_exact() {
    let result = Decimal::from_scientific_exact("1.2345E-28");
    assert!(result.is_err());
    assert_eq!(
        result.err(),
        Some(Error::ScaleExceedsMaximumPrecision(32)) // 4 + 28
    );

    let result = Decimal::from_scientific_exact("12345E29");
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::ScaleExceedsMaximumPrecision(29)));

    let result = Decimal::from_scientific_exact("12345E28");
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::ExceedsMaximumPossibleValue));
}

#[test]
fn it_can_parse_scientific_notation_rounded() {
    let tests = &[
        ("9.7e-7", Ok("0.00000097".to_string())),
        ("9e-7", Ok("0.0000009".to_string())),
        ("1.2e10", Ok("12000000000".to_string())),
        ("1.2e+10", Ok("12000000000".to_string())),
        ("12e10", Ok("120000000000".to_string())),
        ("9.7E-7", Ok("0.00000097".to_string())),
        ("1.2345E-24", Ok("0.0000000000000000000000012345".to_string())),
        ("12345E-28", Ok("0.0000000000000000000000012345".to_string())),
        ("1.2345E0", Ok("1.2345".to_string())),
        ("1E28", Ok("10000000000000000000000000000".to_string())),
        ("1.2345E-28", Ok("0.0000000000000000000000000001".to_string())),
        ("8.7654E-28", Ok("0.0000000000000000000000000009".to_string())),
        (
            "-20165.4676_e-+4294967292",
            Err(Error::ScaleExceedsMaximumPrecision(4294967292)),
        ),
    ];

    for &(value, ref expected) in tests {
        let actual = Decimal::from_scientific_lossy(value).map(|d| d.to_string());
        assert_eq!(*expected, actual);
    }
}

#[test]
fn it_errors_parsing_large_scientific_notation_rounded() {
    let result = Decimal::from_scientific_lossy("1.2345E-29");
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::ScaleExceedsMaximumPrecision(29)));

    let result = Decimal::from_scientific_lossy("12345E29");
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::ScaleExceedsMaximumPrecision(29)));

    let result = Decimal::from_scientific_lossy("12345E28");
    assert!(result.is_err());
    assert_eq!(result.err(), Some(Error::ExceedsMaximumPossibleValue));
}

#[test]
fn it_can_parse_different_radix() {
    let tests = &[
        // Input, Radix, Success, to_string()
        ("123", 10, true, "123"),
        ("123", 8, true, "83"),
        ("123", 16, true, "291"),
        ("abc", 10, false, ""),
        ("abc", 16, true, "2748"),
        ("78", 10, true, "78"),
        ("78", 8, false, ""),
        ("101", 2, true, "5"),
        // Parse base 2
        ("1111_1111_1111_1111_1111_1111_1111_1111", 2, true, "4294967295"),
        // Max supported value
        (
            "1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_\
          1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111",
            2,
            true,
            &Decimal::MAX.to_string(),
        ),
        // We limit to 28 dp
        (
            "843.6500000000000000000000000000",
            10,
            true,
            "843.6500000000000000000000000",
        ),
    ];

    for &(input, radix, success, expected) in tests {
        let result = Decimal::from_str_radix(input, radix);
        assert_eq!(
            success,
            result.is_ok(),
            "Failed to parse: {} radix {}: {:?}",
            input,
            radix,
            result.err()
        );
        if result.is_ok() {
            assert_eq!(
                expected,
                result.unwrap().to_string(),
                "Original input: {input} radix {radix}"
            );
        }
    }
}

#[test]
fn from_str_radix_base2_97_ones_returns_error_not_panic() {
    // 97 binary ones → value exceeds Decimal::MAX; overflow check was after push so panicked
    let result = Decimal::from_str_radix(&"1".repeat(97), 2);
    assert!(result.is_err(), "expected Err, got {result:?}");
}

#[test]
fn from_str_radix_base2_97_leading_zeros_then_one_returns_one() {
    // 97 leading zeros + "1" → value = 1; leading zeros exhausted capacity so panicked
    let s = format!("{}1", "0".repeat(97));
    assert_eq!(Decimal::from_str_radix(&s, 2), Ok(Decimal::ONE));
}

#[test]
fn from_str_radix_base2_30_fractional_digits_returns_error_not_panic() {
    // 30 binary fractional digits → scale = 30 > MAX_SCALE; triggers assert in from_parts
    let result = Decimal::from_str_radix("0.000000000000000000000000000001", 2);
    assert!(result.is_err(), "expected Err, got {result:?}");
}

#[test]
fn from_str_radix_base16_leading_zeros_do_not_corrupt_magnitude() {
    // 24 leading zeros fill the 24-digit hex precision budget; "98" then gets mangled
    // 0x98 = 152 decimal; previously returned Ok(16) due to dropped significant digits
    let s = format!("{}98", "0".repeat(24));
    assert_eq!(
        Decimal::from_str_radix(&s, 16),
        Ok(Decimal::from(152u32)),
        "0x98 should parse as 152"
    );
}
