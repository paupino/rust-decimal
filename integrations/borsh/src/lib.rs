//! [Borsh](https://borsh.io/) serialization for [`rust_decimal::Decimal`] via the [`BorshDecimal`]
//! wrapper. The wire format and schema match the former `borsh` feature of `rust_decimal`.
//!
//! ```
//! use borsh::{BorshDeserialize, BorshSerialize};
//! use rust_decimal::Decimal;
//! use rust_decimal_borsh::BorshDecimal;
//!
//! #[derive(BorshSerialize, BorshDeserialize)]
//! struct Order {
//!     amount: BorshDecimal,
//! }
//!
//! let order = Order { amount: Decimal::new(1234, 2).into() };
//! let bytes = borsh::to_vec(&order).unwrap();
//! let order = Order::try_from_slice(&bytes).unwrap();
//! assert_eq!(Decimal::from(order.amount), Decimal::new(1234, 2));
//! ```
#![forbid(unsafe_code)]

use borsh::schema::{Declaration, Definition, Fields, add_definition};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use rust_decimal::{Decimal, Error};
use std::collections::BTreeMap;
use std::io;

const SIGN_MASK: u32 = 0x8000_0000;
const SCALE_MASK: u32 = 0x00FF_0000;
const SCALE_SHIFT: u32 = 16;

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
        // `Decimal::serialize` is little-endian `flags, lo, mid, hi`; Borsh writes the struct
        // field order `flags, hi, lo, mid`.
        let b = self.0.serialize();
        writer.write_all(&b[0..4])?;
        writer.write_all(&b[12..16])?;
        writer.write_all(&b[4..12])
    }
}

impl BorshDeserialize for BorshDecimal {
    /// Checks the received data to ensure it is a valid [`Decimal`].
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
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
        Ok(Self(Decimal::from_parts(lo, mid, hi, flags & SIGN_MASK != 0, scale)))
    }
}

impl BorshSchema for BorshDecimal {
    fn add_definitions_recursively(definitions: &mut BTreeMap<Declaration, Definition>) {
        let fields = Fields::NamedFields(
            ["flags", "hi", "lo", "mid"]
                .into_iter()
                .map(|name| (name.into(), u32::declaration()))
                .collect(),
        );
        add_definition(Self::declaration(), Definition::Struct { fields }, definitions);
        u32::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        "Decimal".into()
    }
}
