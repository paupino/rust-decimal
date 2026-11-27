use core::str::FromStr;
use rust_decimal::Decimal;

#[test]
fn it_formats() {
    let a = Decimal::from_str("233.323223").unwrap();
    assert_eq!(format!("{a}"), "233.323223");
    assert_eq!(format!("{a:.9}"), "233.323223000");
    assert_eq!(format!("{a:.0}"), "233");
    assert_eq!(format!("{a:.2}"), "233.32");
    assert_eq!(format!("{a:010.2}"), "0000233.32");
    assert_eq!(format!("{a:0<10.2}"), "233.320000");
}
#[test]
fn it_formats_neg() {
    let a = Decimal::from_str("-233.323223").unwrap();
    assert_eq!(format!("{a}"), "-233.323223");
    assert_eq!(format!("{a:.9}"), "-233.323223000");
    assert_eq!(format!("{a:.0}"), "-233");
    assert_eq!(format!("{a:.2}"), "-233.32");
    assert_eq!(format!("{a:010.2}"), "-000233.32");
    assert_eq!(format!("{a:0<10.2}"), "-233.32000");
}
#[test]
fn it_formats_small() {
    let a = Decimal::from_str("0.2223").unwrap();
    assert_eq!(format!("{a}"), "0.2223");
    assert_eq!(format!("{a:.9}"), "0.222300000");
    assert_eq!(format!("{a:.0}"), "0");
    assert_eq!(format!("{a:.2}"), "0.22");
    assert_eq!(format!("{a:010.2}"), "0000000.22");
    assert_eq!(format!("{a:0<10.2}"), "0.22000000");
}
#[test]
fn it_formats_small_leading_zeros() {
    let a = Decimal::from_str("0.0023554701772169").unwrap();
    assert_eq!(format!("{a}"), "0.0023554701772169");
    assert_eq!(format!("{a:.9}"), "0.002355470");
    assert_eq!(format!("{a:.0}"), "0");
    assert_eq!(format!("{a:.2}"), "0.00");
    assert_eq!(format!("{a:010.2}"), "0000000.00");
    assert_eq!(format!("{a:0<10.2}"), "0.00000000");
}
#[test]
fn it_formats_small_neg() {
    let a = Decimal::from_str("-0.2223").unwrap();
    assert_eq!(format!("{a}"), "-0.2223");
    assert_eq!(format!("{a:.9}"), "-0.222300000");
    assert_eq!(format!("{a:.0}"), "-0");
    assert_eq!(format!("{a:.2}"), "-0.22");
    assert_eq!(format!("{a:010.2}"), "-000000.22");
    assert_eq!(format!("{a:0<10.2}"), "-0.2200000");
}

#[test]
fn it_formats_zero() {
    let a = Decimal::from_str("0").unwrap();
    assert_eq!(format!("{a}"), "0");
    assert_eq!(format!("{a:.9}"), "0.000000000");
    assert_eq!(format!("{a:.0}"), "0");
    assert_eq!(format!("{a:.2}"), "0.00");
    assert_eq!(format!("{a:010.2}"), "0000000.00");
    assert_eq!(format!("{a:0<10.2}"), "0.00000000");
}

#[test]
fn it_formats_int() {
    let a = Decimal::from_str("5").unwrap();
    assert_eq!(format!("{a}"), "5");
    assert_eq!(format!("{a:.9}"), "5.000000000");
    assert_eq!(format!("{a:.0}"), "5");
    assert_eq!(format!("{a:.2}"), "5.00");
    assert_eq!(format!("{a:010.2}"), "0000005.00");
    assert_eq!(format!("{a:0<10.2}"), "5.00000000");
}

#[test]
fn it_formats_mantissas_around_64_bits() {
    for (mantissa, scale, expected) in [
        (u128::from(u64::MAX), 0, "18446744073709551615"),
        (1u128 << 64, 0, "18446744073709551616"),
        (1u128 << 64, 10, "1844674407.3709551616"),
        // nine digits from the 96 bit part which are all zeros, or start with zeros
        ((1u128 << 64) * 1_000_000_000, 0, "18446744073709551616000000000"),
        ((1u128 << 64) * 1_000_000_000 + 5, 0, "18446744073709551616000000005"),
        ((1u128 << 64) * 1_000_000_000 + 5, 28, "1.8446744073709551616000000005"),
        (1, 28, "0.0000000000000000000000000001"),
    ] {
        let (lo, mid, hi) = (mantissa as u32, (mantissa >> 32) as u32, (mantissa >> 64) as u32);
        let positive = Decimal::from_parts(lo, mid, hi, false, scale);
        let negative = Decimal::from_parts(lo, mid, hi, true, scale);
        assert_eq!(positive.to_string(), expected);
        assert_eq!(negative.to_string(), format!("-{expected}"));
    }
}

#[test]
fn it_formats_max_and_min() {
    assert_eq!(Decimal::MAX.to_string(), "79228162514264337593543950335");
    assert_eq!(Decimal::MIN.to_string(), "-79228162514264337593543950335");
    let max_scaled = Decimal::from_parts(u32::MAX, u32::MAX, u32::MAX, false, 28);
    assert_eq!(max_scaled.to_string(), "7.9228162514264337593543950335");
}

#[test]
fn it_formats_96_bit_mantissas_with_precision() {
    // 1844674407.3709551616
    let a = Decimal::from_parts(0, 0, 1, false, 10);
    assert_eq!(format!("{a:.0}"), "1844674407");
    assert_eq!(format!("{a:.3}"), "1844674407.370");
    assert_eq!(format!("{a:.12}"), "1844674407.370955161600");
    assert_eq!(format!("{a:>25.3}"), "           1844674407.370");
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn it_formats_lower_exp() {
    let tests = [
        ("0.00001", "1e-5"),
        ("-0.00001", "-1e-5"),
        ("42.123", "4.2123e1"),
        ("-42.123", "-4.2123e1"),
        ("100", "1e2"),
    ];
    for (value, expected) in &tests {
        let a = Decimal::from_str(value).unwrap();
        assert_eq!(&format!("{a:e}"), *expected, "format!(\"{{:e}}\", {a})");
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn it_formats_lower_exp_padding() {
    let tests = [
        ("0.00001", "01e-5"),
        ("-0.00001", "-1e-5"),
        ("42.123", "4.2123e1"),
        ("-42.123", "-4.2123e1"),
        ("100", "001e2"),
    ];
    for (value, expected) in &tests {
        let a = Decimal::from_str(value).unwrap();
        assert_eq!(&format!("{a:05e}"), *expected, "format!(\"{{:05e}}\", {a})");
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn it_formats_zero_scientific_flags() {
    for scale in [0, 3, 28] {
        let zero = Decimal::new(0, scale);
        assert_eq!(format!("{zero:e}"), "0e0");
        assert_eq!(format!("{zero:E}"), "0E0");
        assert_eq!(format!("{zero:.0e}"), "0e0");
        assert_eq!(format!("{zero:.3e}"), "0.000e0");
        assert_eq!(format!("{zero:+08.2E}"), "+00.00E0");
        assert_eq!(format!("{zero:*>8e}"), "*****0e0");
        let negative_zero = -zero;
        assert_eq!(format!("{negative_zero:.2E}"), "-0.00E0");
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[test]
fn it_formats_scientific_precision() {
    for (num, scale, expected_no_precision, expected_precision) in [
        (
            123456,
            10,
            "1.23456e-5",
            [
                "1e-5",
                "1.2e-5",
                "1.23e-5",
                "1.234e-5",
                "1.2345e-5",
                "1.23456e-5",
                "1.234560e-5",
                "1.2345600e-5",
            ],
        ),
        (
            123456,
            0,
            "1.23456e5",
            [
                "1e5",
                "1.2e5",
                "1.23e5",
                "1.234e5",
                "1.2345e5",
                "1.23456e5",
                "1.234560e5",
                "1.2345600e5",
            ],
        ),
        (
            1,
            0,
            "1e0",
            [
                "1e0",
                "1.0e0",
                "1.00e0",
                "1.000e0",
                "1.0000e0",
                "1.00000e0",
                "1.000000e0",
                "1.0000000e0",
            ],
        ),
        (
            -123456,
            10,
            "-1.23456e-5",
            [
                "-1e-5",
                "-1.2e-5",
                "-1.23e-5",
                "-1.234e-5",
                "-1.2345e-5",
                "-1.23456e-5",
                "-1.234560e-5",
                "-1.2345600e-5",
            ],
        ),
        (
            -100000,
            10,
            "-1e-5",
            [
                "-1e-5",
                "-1.0e-5",
                "-1.00e-5",
                "-1.000e-5",
                "-1.0000e-5",
                "-1.00000e-5",
                "-1.000000e-5",
                "-1.0000000e-5",
            ],
        ),
    ] {
        assert_eq!(format!("{:e}", Decimal::new(num, scale)), expected_no_precision);
        for (i, precision) in expected_precision.iter().enumerate() {
            assert_eq!(&format!("{:.prec$e}", Decimal::new(num, scale), prec = i), precision);
        }
    }
}
