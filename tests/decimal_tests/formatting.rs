use core::{fmt, fmt::Write, str::FromStr};
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

fn precision_body(value: &str, precision: usize) -> String {
    let value = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let mut body = whole.to_owned();
    if precision > 0 {
        body.push('.');
        body.push_str(&fraction[..fraction.len().min(precision)]);
        body.extend(core::iter::repeat('0').take(precision.saturating_sub(fraction.len())));
    }
    body
}

#[test]
fn it_formats_precision_without_a_buffer_limit() {
    for source in [
        "0",
        "-0",
        "1",
        "-1",
        "1000",
        "12345",
        "12345678",
        "79228162514264337593543950335",
        "-79228162514264337593543950335",
        "233.323223",
        "0.0023554701772169",
        "0.0000000000000000000000000001",
        "7.9228162514264337593543950335",
    ] {
        let mut value = Decimal::from_str(source).unwrap();
        value.set_sign_negative(source.starts_with('-'));
        for precision in [0, 1, 2, 27, 28, 29, 62, 64, 70, 128, 4096] {
            let sign = if source.starts_with('-') { "-" } else { "" };
            let expected = format!("{sign}{}", precision_body(source, precision));
            assert_eq!(
                format!("{value:.precision$}"),
                expected,
                "{source}, precision {precision}"
            );
            assert_eq!(
                format!("{value:.precision$?}"),
                expected,
                "Debug: {source}, precision {precision}"
            );
        }
        assert_eq!(
            value.array_string().as_ref(),
            source.strip_prefix('-').unwrap_or(source)
        );
    }
}

struct IntegralDisplay<'a> {
    positive: bool,
    body: &'a str,
}

impl fmt::Display for IntegralDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad_integral(self.positive, "", self.body)
    }
}

#[test]
fn it_formats_streamed_precision_like_pad_integral() {
    for source in [
        "1",
        "-1",
        "0",
        "-0",
        "1000",
        "233.323223",
        "79228162514264337593543950335",
    ] {
        let mut value = Decimal::from_str(source).unwrap();
        value.set_sign_negative(source.starts_with('-'));
        for precision in [1, 2, 28, 29, 64, 70] {
            let body = precision_body(source, precision);
            let reference = IntegralDisplay {
                positive: !source.starts_with('-'),
                body: &body,
            };
            for width in [0, 1, 64, 72, 81, 128] {
                macro_rules! check {
                    ($format:literal) => {
                        assert_eq!(
                            format!($format, value, width = width, precision = precision),
                            format!($format, reference, width = width, precision = precision),
                            "{}: {source}, precision {precision}, width {width}",
                            $format
                        );
                    };
                }
                check!("{:width$.precision$}");
                check!("{:+width$.precision$}");
                check!("{:<width$.precision$}");
                check!("{:>width$.precision$}");
                check!("{:^width$.precision$}");
                check!("{:🦀^width$.precision$}");
                check!("{:0<width$.precision$}");
                check!("{:0width$.precision$}");
                check!("{:+0width$.precision$}");
                check!("{:*<0width$.precision$}");
                check!("{:🦀^+0width$.precision$}");
                check!("{:#width$.precision$}");
            }
        }
    }
}

#[test]
fn it_propagates_precision_writer_errors() {
    let value = Decimal::ONE;
    let mut output = arrayvec::ArrayString::<72>::new();
    write!(&mut output, "{value:.70}").unwrap();
    assert_eq!(output.as_str(), precision_body("1", 70));

    let mut output = arrayvec::ArrayString::<71>::new();
    assert!(write!(&mut output, "{value:.70}").is_err());

    let mut output = arrayvec::ArrayString::<3>::new();
    assert!(write!(&mut output, "{value:80.70}").is_err());
    assert_eq!(output.as_str(), "   ");

    let mut output = arrayvec::ArrayString::<75>::new();
    assert!(write!(&mut output, "{value:<80.70}").is_err());
    assert_eq!(output.as_str(), format!("{}   ", precision_body("1", 70)));

    let mut output = arrayvec::ArrayString::<3>::new();
    assert!(write!(&mut output, "{:080.70}", -value).is_err());
    assert!(output.as_str().starts_with('-'));
}

#[test]
fn it_streams_large_precision_to_a_counting_writer() {
    struct Counter(usize);
    impl fmt::Write for Counter {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            self.0 += value.len();
            Ok(())
        }
    }
    let mut output = Counter(0);
    let precision = 65_535;
    write!(&mut output, "{:.precision$}", Decimal::MAX).unwrap();
    assert_eq!(output.0, precision + 30);
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
