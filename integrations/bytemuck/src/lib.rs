//! [`bytemuck`] support for [`rust_decimal::Decimal`] via the [`PodDecimal`] wrapper.
//!
//! [`PodDecimal`] is `#[repr(transparent)]` over `Decimal`, so slices can be cast in place with
//! [`bytemuck::TransparentWrapper`]:
//!
//! ```
//! use bytemuck::TransparentWrapper;
//! use rust_decimal::Decimal;
//! use rust_decimal_bytemuck::PodDecimal;
//!
//! let values = [Decimal::ONE, Decimal::TWO];
//! let pods: &[PodDecimal] = PodDecimal::wrap_slice(&values);
//! let bytes: &[u8] = bytemuck::cast_slice(pods);
//! assert_eq!(bytes.len(), 32);
//! ```
#![no_std]

use bytemuck::{Pod, TransparentWrapper, Zeroable};
use rust_decimal::Decimal;

/// A [`Decimal`] that implements [`Pod`] and [`Zeroable`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PodDecimal(pub Decimal);

// Decimal is four `u32` fields; `rust_decimal`'s `c-repr` feature (enabled by this crate) makes
// the layout `repr(C)`, so there is no padding.
const _: () = assert!(core::mem::size_of::<Decimal>() == 16);

// SAFETY: with `c-repr`, `Decimal` is `repr(C)` over four `u32`s: no padding, and every bit
// pattern (including all zeros) is a valid value. `PodDecimal` is `repr(transparent)` over it.
unsafe impl Zeroable for PodDecimal {}
// SAFETY: see above.
unsafe impl Pod for PodDecimal {}
// SAFETY: `PodDecimal` is `repr(transparent)` over `Decimal`.
unsafe impl TransparentWrapper<Decimal> for PodDecimal {}

impl From<Decimal> for PodDecimal {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<PodDecimal> for Decimal {
    fn from(value: PodDecimal) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_bytes() {
        let value = PodDecimal(Decimal::new(-123456, 3));
        let bytes = bytemuck::bytes_of(&value);
        assert_eq!(bytemuck::pod_read_unaligned::<PodDecimal>(bytes), value);
        assert_eq!(PodDecimal::zeroed().0, Decimal::ZERO);
    }
}
