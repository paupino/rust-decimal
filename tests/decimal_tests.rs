extern crate num;
extern crate rust_decimal;

use num::ToPrimitive;
use num::Zero;
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::str::FromStr;

// Parsing

#[test]
fn it_creates_a_new_negative_decimal() {
    let a = Decimal::new(-100, 2);
    assert_eq!(a.is_sign_negative(), true);
    assert_eq!(a.scale(), 2);
    assert_eq!("-1.00", a.to_string());
}

#[test]
fn it_parses_empty_string() {
    assert!(Decimal::from_str("").is_err());
    assert!(Decimal::from_str(" ").is_err());
}

#[test]
fn it_parses_positive_int_string() {
    let a = Decimal::from_str("233").unwrap();
    assert_eq!(a.is_sign_negative(), false);
    assert_eq!(a.scale(), 0);
    assert_eq!("233", a.to_string());
}

#[test]
fn it_parses_negative_int_string() {
    let a = Decimal::from_str("-233").unwrap();
    assert_eq!(a.is_sign_negative(), true);
    assert_eq!(a.scale(), 0);
    println!("to_string");
    assert_eq!("-233", a.to_string());
}

#[test]
fn it_parses_positive_float_string() {
    let a = Decimal::from_str("233.323223").unwrap();
    assert_eq!(a.is_sign_negative(), false);
    assert_eq!(a.scale(), 6);
    assert_eq!("233.323223", a.to_string());
}

#[test]
fn it_parses_negative_float_string() {
    let a = Decimal::from_str("-233.43343").unwrap();
    assert_eq!(a.is_sign_negative(), true);
    assert_eq!(a.scale(), 5);
    assert_eq!("-233.43343", a.to_string());
}

#[test]
fn it_parses_positive_tiny_float_string() {
    let a = Decimal::from_str(".000001").unwrap();
    assert_eq!(a.is_sign_negative(), false);
    assert_eq!(a.scale(), 6);
    assert_eq!("0.000001", a.to_string());
}

#[test]
fn it_parses_negative_tiny_float_string() {
    let a = Decimal::from_str("-0.000001").unwrap();
    assert_eq!(a.is_sign_negative(), true);
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
fn it_can_serialize_deserialize() {
    let a = Decimal::from_str("12.3456789").unwrap();
    let bytes = a.serialize();
    let b = Decimal::deserialize(bytes);
    assert_eq!("12.3456789", b.to_string());
}

// Formatting

#[test]
fn it_formats() {
    let a = Decimal::from_str("233.323223").unwrap();
    assert_eq!(format!("{}", a), "233.323223");
    assert_eq!(format!("{:.9}", a), "233.323223000");
    assert_eq!(format!("{:.0}", a), "233");
    assert_eq!(format!("{:.2}", a), "233.32");
    assert_eq!(format!("{:010.2}", a), "0000233.32");
    assert_eq!(format!("{:0<10.2}", a), "233.320000");
}
#[test]
fn it_formats_neg() {
    let a = Decimal::from_str("-233.323223").unwrap();
    assert_eq!(format!("{}", a), "-233.323223");
    assert_eq!(format!("{:.9}", a), "-233.323223000");
    assert_eq!(format!("{:.0}", a), "-233");
    assert_eq!(format!("{:.2}", a), "-233.32");
    assert_eq!(format!("{:010.2}", a), "-000233.32");
    assert_eq!(format!("{:0<10.2}", a), "-233.32000");
}
#[test]
fn it_formats_small() {
    let a = Decimal::from_str("0.2223").unwrap();
    assert_eq!(format!("{}", a), "0.2223");
    assert_eq!(format!("{:.9}", a), "0.222300000");
    assert_eq!(format!("{:.0}", a), "0");
    assert_eq!(format!("{:.2}", a), "0.22");
    assert_eq!(format!("{:010.2}", a), "0000000.22");
    assert_eq!(format!("{:0<10.2}", a), "0.22000000");
}
#[test]
fn it_formats_small_neg() {
    let a = Decimal::from_str("-0.2223").unwrap();
    assert_eq!(format!("{}", a), "-0.2223");
    assert_eq!(format!("{:.9}", a), "-0.222300000");
    assert_eq!(format!("{:.0}", a), "-0");
    assert_eq!(format!("{:.2}", a), "-0.22");
    assert_eq!(format!("{:010.2}", a), "-000000.22");
    assert_eq!(format!("{:0<10.2}", a), "-0.2200000");
}
#[test]
fn it_formats_zero() {
    let a = Decimal::from_str("0").unwrap();
    assert_eq!(format!("{}", a), "0");
    assert_eq!(format!("{:.9}", a), "0.000000000");
    assert_eq!(format!("{:.0}", a), "0");
    assert_eq!(format!("{:.2}", a), "0.00");
    assert_eq!(format!("{:010.2}", a), "0000000.00");
    assert_eq!(format!("{:0<10.2}", a), "0.00000000");
}
#[test]
fn it_formats_int() {
    let a = Decimal::from_str("5").unwrap();
    assert_eq!(format!("{}", a), "5");
    assert_eq!(format!("{:.9}", a), "5.000000000");
    assert_eq!(format!("{:.0}", a), "5");
    assert_eq!(format!("{:.2}", a), "5.00");
    assert_eq!(format!("{:010.2}", a), "0000005.00");
    assert_eq!(format!("{:0<10.2}", a), "5.00000000");
}


// Addition

#[test]
fn it_adds_decimals() {
    fn add(a: &str, b: &str, c: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        let result = a + b;
        assert_eq!(
            c,
            result.to_string(),
            "{} + {}",
            a.to_string(),
            b.to_string()
        );
        let result = b + a;
        assert_eq!(
            c,
            result.to_string(),
            "{} + {}",
            b.to_string(),
            a.to_string()
        );
    }

    let tests = &[
        ("2", "3", "5"),
        ("2454495034", "3451204593", "5905699627"),
        ("24544.95034", ".3451204593", "24545.2954604593"),
        (".1", ".1", "0.2"),
        (".10", ".1", "0.20"),
        (".1", "-.1", "0.0"),
        ("0", "1.001", "1.001"),
        ("2", "-3", "-1"),
        ("-2", "3", "1"),
        ("-2", "-3", "-5"),
        ("3", "-2", "1"),
        ("-3", "2", "-1"),
        ("1.234", "2.4567", "3.6907"),
        ("11.815126050420168067226890757", "0.6386554621848739495798319328", "12.45378151260504201681"),
    ];
    for &(a, b, c) in tests {
        add(a, b, c);
    }
}

#[test]
fn it_can_addassign() {
    let mut a = Decimal::from_str("1.01").unwrap();
    let b = Decimal::from_str("0.99").unwrap();
    a += b;
    assert_eq!("2.00", a.to_string());
}

// Subtraction

#[test]
fn it_subtracts_decimals() {
    fn sub(a: &str, b: &str, c: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        let result = a - b;
        assert_eq!(
            c,
            result.to_string(),
            "{} - {}",
            a.to_string(),
            b.to_string()
        );
    }

    let tests = &[
        ("2", "3", "-1"),
        ("3451204593", "2323322332", "1127882261"),
        ("24544.95034", ".3451204593", "24544.6052195407"),
        (".1", ".1", "0.0"),
        (".1", "-.1", "0.2"),
        ("1.001", "0", "1.001"),
        ("2", "-3", "5"),
        ("-2", "3", "-5"),
        ("-2", "-3", "1"),
        ("3", "-2", "5"),
        ("-3", "2", "-5"),
        ("1.234", "2.4567", "-1.2227"),
    ];
    for &(a, b, c) in tests {
        sub(a, b, c);
    }
}

#[test]
fn it_can_subassign() {
    let mut a = Decimal::from_str("1.01").unwrap();
    let b = Decimal::from_str("0.51").unwrap();
    a -= b;
    assert_eq!("0.50", a.to_string());
}

// Multiplication


#[test]
fn it_multiplies_decimals() {
    fn mul(a: &str, b: &str, c: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        let result = a * b;
        assert_eq!(
            c,
            result.to_string(),
            "{} * {}",
            a.to_string(),
            b.to_string()
        );
        let result = b * a;
        assert_eq!(
            c,
            result.to_string(),
            "{} * {}",
            b.to_string(),
            a.to_string()
        );
    }

    let tests = &[
        ("2", "3", "6"),
        ("2454495034", "3451204593", "8470964534836491162"),
        ("24544.95034", ".3451204593", "8470.964534836491162"),
        (".1", ".1", "0.01"),
        ("0", "1.001", "0"),
        ("2", "-3", "-6"),
        ("-2", "3", "-6"),
        ("-2", "-3", "6"),
        ("1", "2.01", "2.01"),
        ("1.0", "2.01", "2.010"), // Scale is always additive
    ];
    for &(a, b, c) in tests {
        mul(a, b, c);
    }
}

#[test]
#[should_panic]
fn it_panics_when_multiply_with_underflow() {
    let a = Decimal::from_str("2.0000000000000000000000000001").unwrap();
    let b = a * a;
    println!("{}", b.to_string());
}

#[test]
#[should_panic]
fn it_panics_when_multiply_with_overflow() {
    let a = Decimal::from_str("2000000000000000000001").unwrap();
    let b = Decimal::from_str("3000000000000000000001").unwrap();
    let _ = a * b;
}

#[test]
fn it_can_mulassign() {
    let mut a = Decimal::from_str("1.25").unwrap();
    let b = Decimal::from_str("0.01").unwrap();
    a *= b;
    assert_eq!("0.0125", a.to_string());
}

// Division

#[test]
fn it_divides_decimals() {
    fn div(a: &str, b: &str, c: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        let result = a / b;
        assert_eq!(
            c,
            result.to_string(),
            "{} / {}",
            a.to_string(),
            b.to_string()
        );
    }

    let tests = &[
        ("6", "3", "2"),
        ("10", "2", "5"),
        ("2.2", "1.1", "2"),
        ("-2.2", "-1.1", "2"),
        ("12.88", "5.6", "2.3"),
        (
            "1023427554493",
            "43432632",
            "23563.562864276795382789603908",
        ),
        ("10000", "3", "3333.3333333333333333333333333"),
        ("2", "3", "0.6666666666666666666666666667"),
        ("1", "3", "0.3333333333333333333333333333"),
        ("-2", "3", "-0.6666666666666666666666666667"),
        ("2", "-3", "-0.6666666666666666666666666667"),
        ("-2", "-3", "0.6666666666666666666666666667"),
    ];
    for &(a, b, c) in tests {
        div(a, b, c);
    }
}

#[test]
#[should_panic]
fn it_can_divide_by_zero() {
    let a = Decimal::from_str("2").unwrap();
    let _ = a / Decimal::zero();
}

#[test]
fn it_can_divassign() {
    let mut a = Decimal::from_str("1.25").unwrap();
    let b = Decimal::from_str("0.01").unwrap();
    a /= b;
    assert_eq!("125", a.to_string());
}

// Modulus and Remainder are not the same thing!
// https://math.stackexchange.com/q/801962/82277

#[test]
fn it_rems_decimals() {
    fn rem(a: &str, b: &str, c: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        // a = qb + r
        let result = a % b;
        assert_eq!(
            c,
            result.to_string(),
            "{} % {}",
            a.to_string(),
            b.to_string()
        );
    }

    let tests = &[
        ("2", "3", "2"),
        ("-2", "3", "-2"),
        ("2", "-3", "2"),
        ("-2", "-3", "-2"),
        ("6", "3", "0"),
    ];
    for &(a, b, c) in tests {
        rem(a, b, c);
    }
}

#[test]
fn it_can_remassign() {
    let mut a = Decimal::from_str("5").unwrap();
    let b = Decimal::from_str("2").unwrap();
    a %= b;
    assert_eq!("1", a.to_string());
}

#[test]
fn it_eqs_decimals() {
    fn eq(a: &str, b: &str, c: bool) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        assert_eq!(c, a.eq(&b), "{} == {}", a.to_string(), b.to_string());
        assert_eq!(c, b.eq(&a), "{} == {}", b.to_string(), a.to_string());
    }

    let tests = &[
        ("1", "1", true),
        ("1", "-1", false),
        ("1", "1.00", true),
        ("1.2345000000000", "1.2345", true),
        (
            "1.0000000000000000000000000000",
            "1.0000000000000000000000000000",
            true,
        ),
        (
            "1.0000000000000000000000000001",
            "1.0000000000000000000000000000",
            false,
        ),
    ];
    for &(a, b, c) in tests {
        eq(a, b, c);
    }
}

#[test]
fn it_cmps_decimals() {
    fn cmp(a: &str, b: &str, c: Ordering) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        assert_eq!(c, a.cmp(&b), "{} {:?} {}", a.to_string(), c, b.to_string());
    }

    let tests = &[
        ("1", "1", Equal),
        ("1", "-1", Greater),
        ("1", "1.00", Equal),
        ("1.2345000000000", "1.2345", Equal),
        (
            "1.0000000000000000000000000001",
            "1.0000000000000000000000000000",
            Greater,
        ),
        (
            "1.0000000000000000000000000000",
            "1.0000000000000000000000000001",
            Less,
        ),
        ("-1", "100", Less),
        ("-100", "1", Less),
        ("0", "0.5", Less),
        ("0.5", "0", Greater),
        ("100", "0.0098", Greater),
        ("1000000000000000", "999000000000000.0001", Greater),
        ("2.0001", "2.0001", Equal),
    ];
    for &(a, b, c) in tests {
        cmp(a, b, c);
    }
}

#[test]
fn test_max_compares() {
    let x = "225.33543601344182".parse::<Decimal>().unwrap();
    let y = Decimal::max_value();
    assert!(x < y);
    assert!(y > x);
    assert!(y != x);

}

#[test]
fn test_min_compares() {
    let x = "225.33543601344182".parse::<Decimal>().unwrap();
    let y = Decimal::min_value();
    assert!(x > y);
    assert!(y < x);
    assert!(y != x);
}

#[test]
fn it_can_round_to_2dp() {
    let a = Decimal::from_str("6.12345").unwrap();
    let b = (Decimal::from_str("100").unwrap() * a).round() / Decimal::from_str("100").unwrap();
    assert_eq!("6.12", b.to_string());
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
fn it_can_return_the_max_value() {
    assert_eq!(
        "79228162514264337593543950335",
        Decimal::max_value().to_string()
    );
}

#[test]
fn it_can_return_the_min_value() {
    assert_eq!(
        "-79228162514264337593543950335",
        Decimal::min_value().to_string()
    );
}

#[test]
fn it_can_go_from_and_into() {
    let d = Decimal::from_str("5").unwrap();
    let di8 = 5u8.into();
    let di32 = 5i32.into();
    let disize = 5isize.into();
    let di64 = 5i64.into();
    let du8 = 5u8.into();
    let du32 = 5u32.into();
    let dusize = 5usize.into();
    let du64 = 5u64.into();

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
    assert_eq!(5f64, Decimal::from_str("5").unwrap().to_f64().unwrap());
    assert_eq!(-5f64, Decimal::from_str("-5").unwrap().to_f64().unwrap());
    assert_eq!(0.1f64, Decimal::from_str("0.1").unwrap().to_f64().unwrap());
    assert_eq!(
        0.25e-11f64,
        Decimal::from_str("0.0000000000025")
            .unwrap()
            .to_f64()
            .unwrap()
    );
    assert_eq!(
        1e6f64,
        Decimal::from_str("1000000.0000000000025")
            .unwrap()
            .to_f64()
            .unwrap()
    );
}

#[test]
fn it_converts_to_i64() {
    assert_eq!(5i64, Decimal::from_str("5").unwrap().to_i64().unwrap());
    assert_eq!(-5i64, Decimal::from_str("-5").unwrap().to_i64().unwrap());
    assert_eq!(
        5i64,
        Decimal::from_str("5.12345").unwrap().to_i64().unwrap()
    );
    assert_eq!(
        -5i64,
        Decimal::from_str("-5.12345").unwrap().to_i64().unwrap()
    );
    assert_eq!(
        0x7FFF_FFFF_FFFF_FFFF,
        Decimal::from_str("9223372036854775807")
            .unwrap()
            .to_i64()
            .unwrap()
    );
    assert_eq!(
        None,
        Decimal::from_str("92233720368547758089").unwrap().to_i64()
    );
}

#[test]
fn it_converts_to_u64() {
    assert_eq!(5u64, Decimal::from_str("5").unwrap().to_u64().unwrap());
    assert_eq!(None, Decimal::from_str("-5").unwrap().to_u64());
    assert_eq!(
        5u64,
        Decimal::from_str("5.12345").unwrap().to_u64().unwrap()
    );
    assert_eq!(
        0xFFFF_FFFF_FFFF_FFFF,
        Decimal::from_str("18446744073709551615")
            .unwrap()
            .to_u64()
            .unwrap()
    );
    assert_eq!(
        None,
        Decimal::from_str("18446744073709551616").unwrap().to_u64()
    );
}

#[test]
fn it_converts_from_f32() {
    fn from_f32(f: f32) -> Option<Decimal> {
        num::FromPrimitive::from_f32(f)
    }

    assert_eq!("1", from_f32(1f32).unwrap().to_string());
    assert_eq!("0", from_f32(0f32).unwrap().to_string());
    assert_eq!("0.12345", from_f32(0.12345f32).unwrap().to_string());
    assert_eq!(
        "0.12345678",
        from_f32(0.1234567800123456789012345678f32)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0.12345679",
        from_f32(0.12345678901234567890123456789f32)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0",
        from_f32(0.00000000000000000000000000001f32)
            .unwrap()
            .to_string()
    );

    assert!(from_f32(std::f32::NAN).is_none());
    assert!(from_f32(std::f32::INFINITY).is_none());

    // These both overflow
    assert!(from_f32(std::f32::MAX).is_none());
    assert!(from_f32(std::f32::MIN).is_none());
}

#[test]
fn it_converts_from_f64() {
    fn from_f64(f: f64) -> Option<Decimal> {
        num::FromPrimitive::from_f64(f)
    }

    assert_eq!("1", from_f64(1f64).unwrap().to_string());
    assert_eq!("0", from_f64(0f64).unwrap().to_string());
    assert_eq!("0.12345", from_f64(0.12345f64).unwrap().to_string());
    assert_eq!(
        "0.1234567890123456",
        from_f64(0.1234567890123456089012345678f64)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0.1234567890123457",
        from_f64(0.12345678901234567890123456789f64)
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0",
        from_f64(0.00000000000000000000000000001f64)
            .unwrap()
            .to_string()
    );

    assert!(from_f64(std::f64::NAN).is_none());
    assert!(from_f64(std::f64::INFINITY).is_none());

    // These both overflow
    assert!(from_f64(std::f64::MAX).is_none());
    assert!(from_f64(std::f64::MIN).is_none());
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
fn it_can_parse_highly_significant_numbers() {
    let tests = &[
        (
            "11.111111111111111111111111111",
            "11.111111111111111111111111111",
        ),
        (
            "11.11111111111111111111111111111",
            "11.111111111111111111111111111",
        ),
        (
            "11.1111111111111111111111111115",
            "11.111111111111111111111111112",
        ),
        (
            "115.111111111111111111111111111",
            "115.11111111111111111111111111",
        ),
        (
            "1115.11111111111111111111111111",
            "1115.1111111111111111111111111",
        ),
        (
            "11.1111111111111111111111111195",
            "11.111111111111111111111111120",
        ),
        (
            "99.9999999999999999999999999995",
            "100.00000000000000000000000000",
        ),
        (
            "-11.1111111111111111111111111195",
            "-11.111111111111111111111111120",
        ),
        (
            "-99.9999999999999999999999999995",
            "-100.00000000000000000000000000",
        ),
    ];
    for &(value, expected) in tests {
        assert_eq!(expected, Decimal::from_str(value).unwrap().to_string());
    }
}
