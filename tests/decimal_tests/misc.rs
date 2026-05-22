use core::str::FromStr;
use num_traits::{Inv, Signed, ToPrimitive};
use rust_decimal::Decimal;

#[test]
#[cfg(feature = "c-repr")]
fn layout_is_correct() {
    assert_eq!(std::mem::size_of::<Decimal>(), std::mem::size_of::<u128>());
}

#[test]
#[cfg(feature = "align16")]
fn align_is_correct() {
    assert_eq!(align_of::<Decimal>(), 16);
    assert_eq!(align_of::<Decimal>(), align_of::<u128>());
    assert_eq!(align_of::<Decimal>(), align_of::<i128>());
}

#[test]
fn it_can_extract_the_mantissa() {
    let tests = [
        ("1", 1i128, 0),
        ("1.123456", 1123456i128, 6),
        ("-0.123456", -123456i128, 6),
    ];
    for &(input, mantissa, scale) in &tests {
        let num = Decimal::from_str(input).unwrap();
        assert_eq!(num.mantissa(), mantissa, "Mantissa for {input}");
        assert_eq!(num.scale(), scale, "Scale for {input}");
    }
}

#[test]
fn it_can_return_the_max_value() {
    assert_eq!("79228162514264337593543950335", Decimal::MAX.to_string());
}

#[test]
fn it_can_return_the_min_value() {
    assert_eq!("-79228162514264337593543950335", Decimal::MIN.to_string());
}

#[test]
fn it_can_calculate_signum() {
    let tests = &[("123", 1), ("-123", -1), ("0", 0)];

    for &(input, expected) in tests {
        let input = Decimal::from_str(input).unwrap();
        assert_eq!(expected, input.signum().to_i32().unwrap(), "Input: {input}");
    }
}

#[test]
fn it_can_calculate_abs_sub() {
    let tests = &[
        ("123", "124", 0),
        ("123", "123", 0),
        ("123", "122", 1),
        ("-123", "-124", 1),
        ("-123", "-123", 0),
        ("-123", "-122", 0),
    ];

    for &(input1, input2, expected) in tests {
        let input1 = Decimal::from_str(input1).unwrap();
        let input2 = Decimal::from_str(input2).unwrap();
        assert_eq!(
            expected,
            input1.abs_sub(&input2).to_i32().unwrap(),
            "Input: {input1} {input2}"
        );
    }
}

#[test]
fn declarative_dec_product() {
    let vs = (1..5).map(|i| i.into()).collect::<Vec<Decimal>>();
    let product: Decimal = vs.into_iter().product();
    assert_eq!(product, Decimal::from(24))
}

#[test]
fn declarative_ref_dec_product() {
    let vs = (1..5).map(|i| i.into()).collect::<Vec<Decimal>>();
    let product: Decimal = vs.iter().product();
    assert_eq!(product, Decimal::from(24))
}

#[test]
fn declarative_dec_sum() {
    let vs = (0..10).map(|i| i.into()).collect::<Vec<Decimal>>();
    let sum: Decimal = vs.into_iter().sum();
    assert_eq!(sum, Decimal::from(45))
}

#[test]
fn declarative_ref_dec_sum() {
    let vs = (0..10).map(|i| i.into()).collect::<Vec<Decimal>>();
    let sum: Decimal = vs.iter().sum();
    assert_eq!(sum, Decimal::from(45))
}

#[test]
fn test_constants() {
    assert_eq!("0", Decimal::ZERO.to_string());
    assert_eq!("1", Decimal::ONE.to_string());
    assert_eq!("-1", Decimal::NEGATIVE_ONE.to_string());
    assert_eq!("10", Decimal::TEN.to_string());
    assert_eq!("100", Decimal::ONE_HUNDRED.to_string());
    assert_eq!("1000", Decimal::ONE_THOUSAND.to_string());
    assert_eq!("2", Decimal::TWO.to_string());
}

#[test]
fn test_inv() {
    assert_eq!("0.01", Decimal::ONE_HUNDRED.inv().to_string());
}

#[test]
fn test_is_integer() {
    let tests = &[
        ("0", true),
        ("1", true),
        ("79_228_162_514_264_337_593_543_950_335", true),
        ("1.0", true),
        ("1.1", false),
        ("3.1415926535897932384626433833", false),
        ("3.0000000000000000000000000000", true),
        ("0.400000000", false),
        ("0.4000000000", false),
        ("0.4000000000000000000", false),
        ("0.4000000000000000001", false),
    ];
    for &(raw, integer) in tests {
        let value = Decimal::from_str(raw).unwrap();
        assert_eq!(value.is_integer(), integer, "value: {raw}")
    }
}
