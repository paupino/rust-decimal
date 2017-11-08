extern crate num;
extern crate rust_decimal;

use num::ToPrimitive;
use num::Zero;
use rust_decimal::Decimal;
use std::str::FromStr;

// Parsing

#[test]
fn it_creates_a_new_negative_decimal() {
    let a = Decimal::new(-100, 2);
    assert_eq!(a.is_negative(), true);
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
    assert_eq!(a.is_negative(), false);
    assert_eq!(a.scale(), 0);
    assert_eq!("233", a.to_string());
}

#[test]
fn it_parses_negative_int_string() {
    let a = Decimal::from_str("-233").unwrap();
    assert_eq!(a.is_negative(), true);
    assert_eq!(a.scale(), 0);
    println!("to_string");
    assert_eq!("-233", a.to_string());
}

#[test]
fn it_parses_positive_float_string() {
    let a = Decimal::from_str("233.323223").unwrap();
    assert_eq!(a.is_negative(), false);
    assert_eq!(a.scale(), 6);
    assert_eq!("233.323223", a.to_string());
}

#[test]
fn it_parses_negative_float_string() {
    let a = Decimal::from_str("-233.43343").unwrap();
    assert_eq!(a.is_negative(), true);
    assert_eq!(a.scale(), 5);
    assert_eq!("-233.43343", a.to_string());
}

#[test]
fn it_parses_positive_tiny_float_string() {
    let a = Decimal::from_str(".000001").unwrap();
    assert_eq!(a.is_negative(), false);
    assert_eq!(a.scale(), 6);
    assert_eq!("0.000001", a.to_string());
}

#[test]
fn it_parses_negative_tiny_float_string() {
    let a = Decimal::from_str("-0.000001").unwrap();
    assert_eq!(a.is_negative(), true);
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
fn it_adds_decimal_1() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a + b;
    assert_eq!("5", c.to_string());
}

#[test]
fn it_adds_decimal_2() {
    let a = Decimal::from_str("2454495034").unwrap();
    let b = Decimal::from_str("3451204593").unwrap();
    let c = a + b;
    assert_eq!("5905699627", c.to_string());
}

#[test]
fn it_adds_decimal_3() {
    let a = Decimal::from_str("24544.95034").unwrap();
    let b = Decimal::from_str(".3451204593").unwrap();
    // Do some sanity checks first
    assert_eq!(5, a.scale());
    assert_eq!(true, a.is_positive());
    assert_eq!(10, b.scale());
    assert_eq!(true, b.is_positive());

    // Do the add
    let c = a + b;
    assert_eq!(10, c.scale());
    assert_eq!("24545.2954604593", c.to_string());
}

#[test]
fn it_adds_decimal_4() {
    let a = Decimal::from_str(".1").unwrap();
    let b = Decimal::from_str(".1").unwrap();
    let c = a + b;
    assert_eq!("0.2", c.to_string());
}

#[test]
fn it_adds_decimal_5() {
    let a = Decimal::from_str(".1").unwrap();
    let b = Decimal::from_str("-.1").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(1, c.scale());
    assert_eq!("0.0", c.to_string());
}

#[test]
fn it_adds_decimal_6() {
    let a = Decimal::from_str("0").unwrap();
    let b = Decimal::from_str("1.001").unwrap();
    let c = a + b;
    assert_eq!("1.001", c.to_string());
}

#[test]
fn it_adds_decimal_7() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("-1", c.to_string());
}

#[test]
fn it_adds_decimal_8() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("1", c.to_string());
}

#[test]
fn it_adds_decimal_9() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("-5", c.to_string());
}

#[test]
fn it_adds_decimal_10() {
    let a = Decimal::from_str("3").unwrap();
    let b = Decimal::from_str("-2").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("1", c.to_string());
}

#[test]
fn it_adds_decimal_11() {
    let a = Decimal::from_str("-3").unwrap();
    let b = Decimal::from_str("2").unwrap();
    let c = a + b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("-1", c.to_string());
}

// Subtraction

#[test]
fn it_subs_decimal_1() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a - b;
    assert_eq!("-1", c.to_string());
}

#[test]
fn it_subs_decimal_2() {
    let a = Decimal::from_str("3451204593").unwrap();
    let b = Decimal::from_str("2323322332").unwrap();
    let c = a - b;
    assert_eq!("1127882261", c.to_string());
}

#[test]
fn it_subs_decimal_3() {
    let a = Decimal::from_str("24544.95034").unwrap();
    let b = Decimal::from_str(".3451204593").unwrap();
    // Do some sanity checks first
    assert_eq!(5, a.scale());
    assert_eq!(true, a.is_positive());
    assert_eq!(10, b.scale());
    assert_eq!(true, b.is_positive());

    // Do the add
    let c = a - b;
    assert_eq!(10, c.scale());
    assert_eq!("24544.6052195407", c.to_string());
}

#[test]
fn it_subs_decimal_4() {
    let a = Decimal::from_str(".1").unwrap();
    let b = Decimal::from_str(".1").unwrap();
    let c = a - b;

    // Keeps the precision
    assert_eq!("0.0", c.to_string());
}

#[test]
fn it_subs_decimal_5() {
    let a = Decimal::from_str(".1").unwrap();
    let b = Decimal::from_str("-.1").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(1, c.scale());
    assert_eq!("0.2", c.to_string());
}

#[test]
fn it_subs_decimal_6() {
    let a = Decimal::from_str("1.001").unwrap();
    let b = Decimal::from_str("0").unwrap();
    let c = a - b;
    assert_eq!("1.001", c.to_string());
}

#[test]
fn it_subs_decimal_7() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("5", c.to_string());
}

#[test]
fn it_subs_decimal_8() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("-5", c.to_string());
}

#[test]
fn it_subs_decimal_9() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("1", c.to_string());
}

#[test]
fn it_subs_decimal_10() {
    let a = Decimal::from_str("3").unwrap();
    let b = Decimal::from_str("-2").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("5", c.to_string());
}

#[test]
fn it_subs_decimal_11() {
    let a = Decimal::from_str("-3").unwrap();
    let b = Decimal::from_str("2").unwrap();
    let c = a - b;
    // We keep the scale of 1 as this is the precision
    assert_eq!(0, c.scale());
    assert_eq!("-5", c.to_string());
}

// Multiplication

#[test]
fn it_can_multiply_1() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a * b;
    assert_eq!("6", c.to_string());
}

#[test]
fn it_can_multiply_2() {
    let a = Decimal::from_str("2454495034").unwrap();
    let b = Decimal::from_str("3451204593").unwrap();
    let c = a * b;
    assert_eq!("8470964534836491162", c.to_string());
}

#[test]
fn it_can_multiply_3() {
    let a = Decimal::from_str("24544.95034").unwrap();
    let b = Decimal::from_str(".3451204593").unwrap();
    let c = a * b;
    assert_eq!("8470.964534836491162", c.to_string());
}

#[test]
fn it_can_multiply_4() {
    let a = Decimal::from_str(".1").unwrap();
    let b = Decimal::from_str(".1").unwrap();
    let c = a * b;
    assert_eq!("0.01", c.to_string());
}

#[test]
fn it_can_multiply_5() {
    let a = Decimal::from_str("0").unwrap();
    let b = Decimal::from_str("1.001").unwrap();
    let c = a * b;
    assert_eq!("0", c.to_string());
}

#[test]
fn it_can_multiply_6() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a * b;
    assert_eq!("-6", c.to_string());
}

#[test]
fn it_can_multiply_7() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a * b;
    assert_eq!("-6", c.to_string());
}

#[test]
fn it_can_multiply_8() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a * b;
    assert_eq!("6", c.to_string());
}

#[test]
#[should_panic]
fn it_panics_when_multiply_with_underflow() {
    let a = Decimal::from_str("2.0000000000000000000000000001").unwrap();
    let _ = a * a;
}

#[test]
#[should_panic]
fn it_panics_when_multiply_with_overflow() {
    let a = Decimal::from_str("2000000000000000000001").unwrap();
    let b = Decimal::from_str("3000000000000000000001").unwrap();
    let _ = a * b;
}

// Division

#[test]
fn it_can_divide_1() {
    let a = Decimal::from_str("6").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;
    assert_eq!("2", c.to_string());
}

#[test]
fn it_can_divide_2() {
    let a = Decimal::from_str("10").unwrap();
    let b = Decimal::from_str("2").unwrap();
    let c = a / b;
    assert_eq!("5", c.to_string());
}

#[test]
fn it_can_divide_3() {
    let a = Decimal::from_str("2.2").unwrap();
    let b = Decimal::from_str("1.1").unwrap();
    let c = a / b;
    assert_eq!("2", c.to_string());
}

#[test]
fn it_can_divide_4() {
    let a = Decimal::from_str("-2.2").unwrap();
    let b = Decimal::from_str("-1.1").unwrap();
    let c = a / b;
    assert_eq!("2", c.to_string());
}

#[test]
fn it_can_divide_5() {
    let a = Decimal::from_str("12.88").unwrap();
    let b = Decimal::from_str("5.6").unwrap();
    let c = a / b;
    assert_eq!("2.3", c.to_string());
}

#[test]
fn it_can_divide_6() {
    let a = Decimal::from_str("1023427554493").unwrap();
    let b = Decimal::from_str("43432632").unwrap();
    let c = a / b;
    assert_eq!("23563.562864276795382789603908", c.to_string()); // Rounded
}

#[test]
fn it_can_divide_7() {
    let a = Decimal::from_str("10000").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;

    assert_eq!("3333.3333333333333333333333333", c.to_string());
}

#[test]
fn it_can_divide_8() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;
    assert_eq!("0.66666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_9() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;
    assert_eq!("-0.66666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_10() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a / b;
    assert_eq!("-0.66666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_11() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a / b;
    assert_eq!("0.66666666666666666666666666666", c.to_string());
}

#[test]
#[should_panic]
fn it_can_divide_by_zero() {
    let a = Decimal::from_str("2").unwrap();
    let _ = a / Decimal::zero();
}

// Modulus and Remainder are not the same thing!
// https://math.stackexchange.com/q/801962/82277

#[test]
fn it_can_rem_1() {
    // a = qb + r
    // 2 = 0*3 + 2
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a % b;
    assert_eq!("2", c.to_string());
}

#[test]
fn it_can_rem_2() {
    // a = qb + r
    // -2 = 0*3 + -2
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a % b;
    assert_eq!("-2", c.to_string());
}

#[test]
fn it_can_rem_3() {
    // a = qb + r
    // 2 = 0*-3 + 2
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a % b;
    assert_eq!("2", c.to_string());
}

#[test]
fn it_can_rem_4() {
    // a = qb + r
    // -2 = 0*-3 + -2
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a % b;
    assert_eq!("-2", c.to_string());
}

#[test]
fn it_can_rem_5() {
    // a = qb + r
    // 6 = 2*3 + 0
    let a = Decimal::from_str("6").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a % b;
    assert_eq!("0", c.to_string());
}

#[test]
fn it_can_eq_1() {
    let a = Decimal::new(1, 0);
    let b = Decimal::new(1, 0);
    assert_eq!(true, a.eq(&b));
}

#[test]
fn it_can_eq_2() {
    let a = Decimal::new(1, 0);
    let b = Decimal::new(-1, 0);
    assert_eq!(false, a.eq(&b));
}

#[test]
fn it_can_eq_3() {
    let a = Decimal::new(1, 0);
    let b = Decimal::new(100, 2);
    assert_eq!(true, a.eq(&b));
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
