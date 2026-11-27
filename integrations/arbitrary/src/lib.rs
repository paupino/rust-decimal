//! [`arbitrary`] support for [`rust_decimal::Decimal`].
//!
//! Use [`arbitrary_decimal`] with `#[arbitrary(with = ...)]`:
//!
//! ```
//! use arbitrary::Arbitrary;
//! use rust_decimal::Decimal;
//!
//! #[derive(Arbitrary, Debug)]
//! struct Input {
//!     #[arbitrary(with = rust_decimal_arbitrary::arbitrary_decimal)]
//!     amount: Decimal,
//! }
//! ```
#![no_std]
#![forbid(unsafe_code)]

use arbitrary::{Arbitrary, Result, Unstructured};
use rust_decimal::Decimal;

/// Generates an arbitrary [`Decimal`] covering the full mantissa and scale range.
pub fn arbitrary_decimal(u: &mut Unstructured<'_>) -> Result<Decimal> {
    let lo = u32::arbitrary(u)?;
    let mid = u32::arbitrary(u)?;
    let hi = u32::arbitrary(u)?;
    let negative = bool::arbitrary(u)?;
    let scale = u32::arbitrary(u)? % (Decimal::MAX_SCALE + 1);
    Ok(Decimal::from_parts(lo, mid, hi, negative, scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_generate_arbitrary_decimals() {
        let mut u = Unstructured::new(b"it_can_generate_arbitrary_decimals");
        assert!(arbitrary_decimal(&mut u).is_ok());
    }
}
