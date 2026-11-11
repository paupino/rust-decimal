use std::str::FromStr;

use rust_decimal::Decimal;

const VALUES: [&str; 6] = [
    "0",
    "-0.00",
    "12.3456789",
    "-5233.9008808150288439427720175",
    "79228162514264337593543950335",
    "0.0000000000000000000000000001",
];

#[test]
fn a_decimal_can_be_viewed_as_bytes() {
    for value in VALUES {
        let decimal = Decimal::from_str(value).unwrap();
        let bytes: [u8; 16] = bytemuck::cast(decimal);
        assert_eq!(bytemuck::bytes_of(&decimal), &bytes, "{value}");
        // The fields are in `repr(C)` order: flags, hi, lo, mid.
        let words: [u32; 4] = bytemuck::cast(decimal);
        assert_eq!(words[0] >> 16 & 0xFF, decimal.scale(), "{value}");
        assert_eq!(words[0] >> 31 == 1, decimal.is_sign_negative(), "{value}");
    }

    let decimals = [Decimal::ONE, Decimal::TWO];
    assert_eq!(bytemuck::cast_slice::<Decimal, u8>(&decimals).len(), 32);
}

#[test]
fn zeroed_is_zero() {
    let zero: Decimal = bytemuck::Zeroable::zeroed();
    assert_eq!(zero, Decimal::ZERO);
    assert_eq!(zero.scale(), 0);
}

/// Bytes become a `Decimal` through `Decimal::deserialize`, which clears the unused flag bits and
/// brings the scale back to at most 28, instead of a cast that would keep them.
#[test]
fn bytes_become_a_decimal_through_deserialize() {
    for value in VALUES {
        let decimal = Decimal::from_str(value).unwrap();
        let back = Decimal::deserialize(decimal.serialize());
        assert_eq!(back, decimal, "{value}");
        assert_eq!(back.scale(), decimal.scale(), "{value}");
    }

    // Scale 31 and the unused flag bit 30, which a cast used to accept as is.
    let mut bytes = Decimal::ONE.serialize();
    bytes[2] = 31;
    bytes[3] = 0x40;
    let decimal = Decimal::deserialize(bytes);
    assert!(decimal.scale() <= Decimal::MAX_SCALE);
    assert_eq!(decimal.serialize()[3] & 0x7F, 0);
    // Formatting works on the result.
    let _ = decimal.to_string();
}
