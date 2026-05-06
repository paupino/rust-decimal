#![cfg(target_arch = "wasm32")]

// Tests use both `Decimal::from_str` (the standard `FromStr` trait) and `Decimal::from_string`
// (the wasm-bindgen binding). This is intentional: `from_string` tests exercise the wasm API
// surface, while `from_str` tests verify that core Decimal functionality works under wasm.

use core::str::FromStr;
use rust_decimal::Decimal;
use wasm_bindgen_test::*;

// from_number

#[wasm_bindgen_test]
fn it_converts_positive_integer_from_number() {
    let a = Decimal::from_number(42.0).unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!("42", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_negative_integer_from_number() {
    let a = Decimal::from_number(-123.0).unwrap();
    assert!(a.is_sign_negative());
    assert_eq!("-123", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_zero_from_number() {
    let a = Decimal::from_number(0.0).unwrap();
    assert_eq!("0", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_positive_float_from_number() {
    let a = Decimal::from_number(3.14).unwrap();
    assert_eq!("3.14", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_small_fractional_from_number() {
    let a = Decimal::from_number(0.001).unwrap();
    assert_eq!("0.001", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_large_number_from_number() {
    let a = Decimal::from_number(1_000_000_000.0).unwrap();
    assert_eq!("1000000000", a.to_string());
}

#[wasm_bindgen_test]
fn it_returns_none_for_nan() {
    assert!(Decimal::from_number(f64::NAN).is_none());
}

#[wasm_bindgen_test]
fn it_returns_none_for_positive_infinity() {
    assert!(Decimal::from_number(f64::INFINITY).is_none());
}

#[wasm_bindgen_test]
fn it_returns_none_for_negative_infinity() {
    assert!(Decimal::from_number(f64::NEG_INFINITY).is_none());
}

// from_string

#[wasm_bindgen_test]
fn it_converts_positive_float_from_string() {
    let a = Decimal::from_string("233.323223").unwrap();
    assert!(!a.is_sign_negative());
    assert_eq!(a.scale(), 6);
    assert_eq!("233.323223", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_negative_float_from_string() {
    let a = Decimal::from_string("-233.43343").unwrap();
    assert!(a.is_sign_negative());
    assert_eq!("-233.43343", a.to_string());
}

#[wasm_bindgen_test]
fn it_converts_big_integer_from_string() {
    let a = Decimal::from_string("79228162514264337593543950330").unwrap();
    assert_eq!("79228162514264337593543950330", a.to_string());
}

#[wasm_bindgen_test]
fn it_returns_none_for_empty_string() {
    assert!(Decimal::from_string("").is_none());
}

#[wasm_bindgen_test]
fn it_returns_none_for_invalid_string() {
    assert!(Decimal::from_string("not_a_number").is_none());
}

// to_string_js

#[wasm_bindgen_test]
fn it_converts_decimal_to_string() {
    let a = Decimal::from_string("12.3456789").unwrap();
    assert_eq!("12.3456789", a.to_string_js());
}

#[wasm_bindgen_test]
fn it_converts_negative_decimal_to_string() {
    let a = Decimal::from_number(-42.0).unwrap();
    assert_eq!("-42", a.to_string_js());
}

// to_number

#[wasm_bindgen_test]
fn it_converts_positive_decimal_to_number() {
    let a = Decimal::from_str("233.323223").unwrap();
    let n = a.to_number();
    assert!((n - 233.323223).abs() < 1e-10);
}

#[wasm_bindgen_test]
fn it_converts_negative_decimal_to_number() {
    let a = Decimal::from_str("-233.43343").unwrap();
    let n = a.to_number();
    assert!((n - -233.43343).abs() < 1e-10);
}

#[wasm_bindgen_test]
fn it_converts_zero_to_number() {
    let a = Decimal::from_str("0").unwrap();
    assert_eq!(a.to_number(), 0.0);
}

// Round-trip: from_number -> to_number

#[wasm_bindgen_test]
fn it_round_trips_positive_float() {
    let n = 12345.6789;
    let a = Decimal::from_number(n).unwrap();
    assert!((a.to_number() - n).abs() < 1e-10);
}

#[wasm_bindgen_test]
fn it_round_trips_negative_float() {
    let n = -9876.54321;
    let a = Decimal::from_number(n).unwrap();
    assert!((a.to_number() - n).abs() < 1e-10);
}

// Round-trip: from_str -> to_number -> from_number -> to_string

#[wasm_bindgen_test]
fn it_round_trips_through_string_and_number() {
    let tests = ["1.5", "100", "0.001", "-42.42"];
    for test in &tests {
        let a = Decimal::from_str(test).unwrap();
        let n = a.to_number();
        let b = Decimal::from_number(n).unwrap();
        assert_eq!(test.to_string(), b.to_string(), "Round-trip failed for {test}");
    }
}

// Round-trip: from_string -> to_string_js

#[wasm_bindgen_test]
fn it_round_trips_from_string_to_string_js() {
    let tests = [
        "12.3456789",
        "79228162514264337593543950330",
        "-5233.9008808150288439427720175",
        "0",
        "0.001",
    ];
    for test in &tests {
        let a = Decimal::from_string(test).unwrap();
        assert_eq!(test.to_string(), a.to_string_js(), "Round-trip failed for {test}");
    }
}

// Round-trip: from_string -> to_number

#[wasm_bindgen_test]
fn it_round_trips_from_string_to_number() {
    let tests = [("1.5", 1.5), ("100", 100.0), ("-42.42", -42.42), ("0.001", 0.001)];
    for (input, expected) in &tests {
        let a = Decimal::from_string(input).unwrap();
        let n = a.to_number();
        assert!((n - expected).abs() < 1e-10, "Round-trip failed for {input}: got {n}");
    }
}

// Arithmetic via wasm-constructed decimals
//
// Note: `from_number` converts via `f64`, which may introduce trailing scale
// (e.g. `1.5` has scale 1). Arithmetic preserves scale from the operands, so
// results may include trailing zeros (e.g. `1.5 + 2.5 = "4.0"` not `"4"`).

#[wasm_bindgen_test]
fn it_can_add_wasm_decimals() {
    let a = Decimal::from_number(1.5).unwrap();
    let b = Decimal::from_number(2.5).unwrap();
    assert_eq!("4.0", (a + b).to_string());
}

#[wasm_bindgen_test]
fn it_can_subtract_wasm_decimals() {
    let a = Decimal::from_number(10.0).unwrap();
    let b = Decimal::from_number(3.5).unwrap();
    assert_eq!("6.5", (a - b).to_string());
}

#[wasm_bindgen_test]
fn it_can_multiply_wasm_decimals() {
    // Multiplication of two integer-valued f64s (scale 0) produces scale 0
    let a = Decimal::from_number(6.0).unwrap();
    let b = Decimal::from_number(7.0).unwrap();
    assert_eq!("42", (a * b).to_string());
}

#[wasm_bindgen_test]
fn it_can_divide_wasm_decimals() {
    // Division preserves scale from operands, so 10.0 / 4.0 yields "2.50"
    let a = Decimal::from_number(10.0).unwrap();
    let b = Decimal::from_number(4.0).unwrap();
    assert_eq!("2.50", (a / b).to_string());
}

// Core functionality works under wasm

#[wasm_bindgen_test]
fn it_can_parse_string_in_wasm() {
    let a = Decimal::from_str("79228162514264337593543950330").unwrap();
    assert_eq!("79228162514264337593543950330", a.to_string());
}

#[wasm_bindgen_test]
fn it_can_serialize_deserialize_in_wasm() {
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

#[wasm_bindgen_test]
fn it_can_compare_wasm_decimals() {
    let a = Decimal::from_number(1.0).unwrap();
    let b = Decimal::from_number(2.0).unwrap();
    let c = Decimal::from_number(1.0).unwrap();
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, c);
}

#[wasm_bindgen_test]
fn it_can_check_sign_of_wasm_decimal() {
    let pos = Decimal::from_number(5.0).unwrap();
    let neg = Decimal::from_number(-5.0).unwrap();
    assert!(!pos.is_sign_negative());
    assert!(neg.is_sign_negative());
}

#[wasm_bindgen_test]
fn it_can_check_zero_of_wasm_decimal() {
    let zero = Decimal::from_number(0.0).unwrap();
    let non_zero = Decimal::from_number(1.0).unwrap();
    assert!(zero.is_zero());
    assert!(!non_zero.is_zero());
}

#[wasm_bindgen_test]
fn it_can_round_wasm_decimal() {
    let a = Decimal::from_str("2.567").unwrap();
    let rounded = a.round_dp(2);
    assert_eq!("2.57", rounded.to_string());
}

#[wasm_bindgen_test]
fn it_can_floor_and_ceil_wasm_decimal() {
    let a = Decimal::from_str("2.3").unwrap();
    assert_eq!("2", a.floor().to_string());
    assert_eq!("3", a.ceil().to_string());
}

#[wasm_bindgen_test]
fn it_can_extract_mantissa_and_scale_in_wasm() {
    let a = Decimal::from_str("1.123456").unwrap();
    assert_eq!(a.mantissa(), 1123456i128);
    assert_eq!(a.scale(), 6);
}

#[wasm_bindgen_test]
fn it_can_access_constants_in_wasm() {
    assert_eq!("0", Decimal::ZERO.to_string());
    assert_eq!("1", Decimal::ONE.to_string());
    assert_eq!("10", Decimal::TEN.to_string());
}
