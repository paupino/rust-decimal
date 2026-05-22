use std::str::FromStr;

use rust_decimal::Decimal;

#[test]
fn it_can_serialize_deserialize_borsh() {
    let tests = [
        "12.3456789",
        "5233.9008808150288439427720175",
        "-5233.9008808150288439427720175",
    ];
    for test in &tests {
        let a = Decimal::from_str(test).unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        borsh::BorshSerialize::serialize(&a, &mut bytes).unwrap();
        let b: Decimal = borsh::BorshDeserialize::deserialize(&mut bytes.as_slice()).unwrap();
        assert_eq!(test.to_string(), b.to_string());
        let bytes = borsh::try_to_vec_with_schema(&a);
        assert!(bytes.is_ok(), "try_to_vec_with_schema.is_ok()");
        let bytes = bytes.unwrap();
        let result = borsh::try_from_slice_with_schema(&bytes);
        assert!(result.is_ok(), "try_from_slice_with_schema.is_ok()");
        let b: Decimal = result.unwrap();
        assert_eq!(test.to_string(), b.to_string());
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

    let _err =
        <Decimal as borsh::BorshDeserialize>::deserialize(&mut bytes.as_slice()).expect_err("Invalid flags passed");
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

    let err =
        <Decimal as borsh::BorshDeserialize>::deserialize(&mut bytes.as_slice()).expect_err("Invalid scale passed");
    assert_eq!(
        err.downcast::<rust_decimal::Error>().expect("Expected str flags error"),
        rust_decimal::Error::ScaleExceedsMaximumPrecision(0xFF)
    );
}
