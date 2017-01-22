extern crate num;
extern crate rust_decimal;

use num::Zero;
use rust_decimal::Decimal;
use std::str::FromStr;

// Parsing

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
    assert_eq!("0.000", c.to_string());
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
    assert_eq!("23563.56286427679538278960390", c.to_string());// Rounded
}

#[test]
fn it_can_divide_7() {
    let a = Decimal::from_str("10000").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;

    assert_eq!("3333.333333333333333333333333", c.to_string());
}

#[test]
fn it_can_divide_8() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;
    assert_eq!("0.6666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_9() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("3").unwrap();
    let c = a / b;
    assert_eq!("-0.6666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_10() {
    let a = Decimal::from_str("2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a / b;
    assert_eq!("-0.6666666666666666666666666666", c.to_string());
}

#[test]
fn it_can_divide_11() {
    let a = Decimal::from_str("-2").unwrap();
    let b = Decimal::from_str("-3").unwrap();
    let c = a / b;
    assert_eq!("0.6666666666666666666666666666", c.to_string());
}

#[test]
#[should_panic]
fn it_can_divide_by_zero() {
    let a = Decimal::from_str("2").unwrap();
    let _ = a / Decimal::zero();
}

// Modulus and Remainder are not the same thing!
// http://math.stackexchange.com/questions/801962/difference-between-modulus-and-remainder?newreg=5c10dc7c34294664ab08fadcbb223545

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
    assert_eq!("79228162514264337593543950335", Decimal::max_value().to_string());
}

#[test]
fn it_can_return_the_min_value() {
    assert_eq!("-79228162514264337593543950335", Decimal::min_value().to_string());
}
