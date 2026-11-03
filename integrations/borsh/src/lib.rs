//! [Borsh](https://borsh.io/) serialization for [`rust_decimal::Decimal`]. The wire format and
//! schema match the former `borsh` feature of `rust_decimal`.
//!
//! Keep `Decimal` fields and point borsh at this crate's functions:
//!
//! ```
//! use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
//! use rust_decimal::Decimal;
//!
//! #[derive(BorshSerialize, BorshDeserialize, BorshSchema)]
//! struct Order {
//!     #[borsh(
//!         serialize_with = "rust_decimal_borsh::serialize",
//!         deserialize_with = "rust_decimal_borsh::deserialize",
//!         // Only needed when deriving `BorshSchema`.
//!         schema(with_funcs(
//!             declaration = "rust_decimal_borsh::schema_declaration",
//!             definitions = "rust_decimal_borsh::schema_definitions"
//!         ))
//!     )]
//!     amount: Decimal,
//! }
//!
//! let order = Order { amount: Decimal::new(1234, 2) };
//! let bytes = borsh::to_vec(&order).unwrap();
//! assert_eq!(Order::try_from_slice(&bytes).unwrap().amount, Decimal::new(1234, 2));
//! ```
//!
//! For values that are not struct fields (e.g. `Vec<Decimal>` or a top-level value), use the
//! [`BorshDecimal`] wrapper, which implements the Borsh traits directly.
#![forbid(unsafe_code)]

use borsh::schema::{Declaration, Definition, Fields, add_definition};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use rust_decimal::{Decimal, Error};
use std::collections::BTreeMap;
use std::io;

const SIGN_MASK: u32 = 0x8000_0000;
const SCALE_MASK: u32 = 0x00FF_0000;
const SCALE_SHIFT: u32 = 16;

/// Serializes a [`Decimal`]. For use with `#[borsh(serialize_with = "...")]`.
pub fn serialize<W: io::Write>(value: &Decimal, writer: &mut W) -> io::Result<()> {
    // `Decimal::serialize` is little-endian `flags, lo, mid, hi`; Borsh writes the struct field
    // order `flags, hi, lo, mid`.
    let b = value.serialize();
    writer.write_all(&b[0..4])?;
    writer.write_all(&b[12..16])?;
    writer.write_all(&b[4..12])
}

/// Deserializes a [`Decimal`], checking that the data is a valid `Decimal`. For use with
/// `#[borsh(deserialize_with = "...")]`.
pub fn deserialize<R: io::Read>(reader: &mut R) -> io::Result<Decimal> {
    let flags = u32::deserialize_reader(reader)?;
    if flags & (SCALE_MASK | SIGN_MASK) != flags {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid flag representation",
        ));
    }
    let scale = (flags & SCALE_MASK) >> SCALE_SHIFT;
    if scale > Decimal::MAX_SCALE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            Error::ScaleExceedsMaximumPrecision(scale),
        ));
    }
    let hi = u32::deserialize_reader(reader)?;
    let lo = u32::deserialize_reader(reader)?;
    let mid = u32::deserialize_reader(reader)?;
    Ok(Decimal::from_parts(lo, mid, hi, flags & SIGN_MASK != 0, scale))
}

/// The schema declaration for a [`Decimal`]. For use with
/// `#[borsh(schema(with_funcs(declaration = "...")))]`.
pub fn schema_declaration() -> Declaration {
    "Decimal".into()
}

/// Adds the schema definitions for a [`Decimal`]. For use with
/// `#[borsh(schema(with_funcs(definitions = "...")))]`.
pub fn schema_definitions(definitions: &mut BTreeMap<Declaration, Definition>) {
    let fields = Fields::NamedFields(
        ["flags", "hi", "lo", "mid"]
            .into_iter()
            .map(|name| (name.into(), u32::declaration()))
            .collect(),
    );
    add_definition(schema_declaration(), Definition::Struct { fields }, definitions);
    u32::add_definitions_recursively(definitions);
}

/// A [`Decimal`] that implements the Borsh traits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BorshDecimal(pub Decimal);

impl From<Decimal> for BorshDecimal {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<BorshDecimal> for Decimal {
    fn from(value: BorshDecimal) -> Self {
        value.0
    }
}

impl BorshSerialize for BorshDecimal {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        serialize(&self.0, writer)
    }
}

impl BorshDeserialize for BorshDecimal {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        deserialize(reader).map(Self)
    }
}

impl BorshSchema for BorshDecimal {
    fn add_definitions_recursively(definitions: &mut BTreeMap<Declaration, Definition>) {
        schema_definitions(definitions)
    }

    fn declaration() -> Declaration {
        schema_declaration()
    }
}
