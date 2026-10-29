use std::str::FromStr;

use rust_decimal::Decimal;
use rust_decimal_borsh::BorshDecimal;

#[test]
fn it_can_serialize_deserialize_borsh() {
    let tests = [
        "12.3456789",
        "5233.9008808150288439427720175",
        "-5233.9008808150288439427720175",
    ];
    for test in &tests {
        let a = BorshDecimal(Decimal::from_str(test).unwrap());
        let mut bytes: Vec<u8> = Vec::new();
        borsh::BorshSerialize::serialize(&a, &mut bytes).unwrap();
        let b: BorshDecimal = borsh::BorshDeserialize::deserialize(&mut bytes.as_slice()).unwrap();
        assert_eq!(test.to_string(), b.0.to_string());
        let bytes = borsh::try_to_vec_with_schema(&a);
        assert!(bytes.is_ok(), "try_to_vec_with_schema.is_ok()");
        let bytes = bytes.unwrap();
        let result = borsh::try_from_slice_with_schema(&bytes);
        assert!(result.is_ok(), "try_from_slice_with_schema.is_ok()");
        let b: BorshDecimal = result.unwrap();
        assert_eq!(test.to_string(), b.0.to_string());
    }
}

#[test]
fn invalid_flags_errors() {
    let mut bytes: Vec<u8> = Vec::new();
    // Invalid flags
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();
    // high
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();
    // lo
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();
    // mid
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();

    let _err = <BorshDecimal as borsh::BorshDeserialize>::deserialize(&mut bytes.as_slice())
        .expect_err("Invalid flags passed");
}

#[test]
fn invalid_scale_errors() {
    let mut bytes: Vec<u8> = Vec::new();
    // Invalid scale
    borsh::BorshSerialize::serialize(&0x00FF_0000_u32, &mut bytes).unwrap();
    // high
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();
    // lo
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();
    // mid
    borsh::BorshSerialize::serialize(&u32::MAX, &mut bytes).unwrap();

    let err = <BorshDecimal as borsh::BorshDeserialize>::deserialize(&mut bytes.as_slice())
        .expect_err("Invalid scale passed");
    assert_eq!(
        err.downcast::<rust_decimal::Error>().expect("Expected str flags error"),
        rust_decimal::Error::ScaleExceedsMaximumPrecision(0xFF)
    );
}

#[test]
fn wire_format_is_flags_hi_lo_mid() {
    // -1.000000000000000000000000001 (scale 27): lo, mid, hi all non-zero.
    let value = BorshDecimal(Decimal::from_parts(1, 2, 3, true, 27));
    let bytes = borsh::to_vec(&value).unwrap();
    assert_eq!(
        bytes,
        [0, 0, 27, 0x80, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0],
        "flags, hi, lo, mid (little-endian)"
    );
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, borsh::BorshSchema, Debug, PartialEq)]
struct Order {
    id: u32,
    #[borsh(
        serialize_with = "rust_decimal_borsh::serialize",
        deserialize_with = "rust_decimal_borsh::deserialize",
        schema(with_funcs(
            declaration = "rust_decimal_borsh::schema_declaration",
            definitions = "rust_decimal_borsh::schema_definitions"
        ))
    )]
    amount: Decimal,
}

#[test]
fn decimal_field_round_trips_with_schema() {
    let order = Order {
        id: 7,
        amount: Decimal::from_str("-5233.9008808150288439427720175").unwrap(),
    };
    let bytes = borsh::to_vec(&order).unwrap();
    assert_eq!(bytes[4..], borsh::to_vec(&BorshDecimal(order.amount)).unwrap());
    assert_eq!(borsh::from_slice::<Order>(&bytes).unwrap(), order);

    let bytes = borsh::try_to_vec_with_schema(&order).unwrap();
    assert_eq!(borsh::try_from_slice_with_schema::<Order>(&bytes).unwrap(), order);
}
