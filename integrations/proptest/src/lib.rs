//! [`proptest`](https://docs.rs/proptest) strategies for [`rust_decimal::Decimal`].
//!
//! ```
//! use proptest::prelude::*;
//! use rust_decimal_proptest::arb_decimal;
//!
//! proptest! {
//!     fn round_trips(d in arb_decimal()) {
//!         prop_assert_eq!(d, d.to_string().parse().unwrap());
//!     }
//! }
//! # round_trips();
//! ```
#![forbid(unsafe_code)]

use proptest::prelude::*;
use rust_decimal::Decimal;

/// A strategy generating any [`Decimal`], covering the full mantissa and scale range.
pub fn arb_decimal() -> impl Strategy<Value = Decimal> {
    (any::<(u32, u32, u32, bool)>(), 0..=Decimal::MAX_SCALE)
        .prop_map(|((lo, mid, hi, negative), scale)| Decimal::from_parts(lo, mid, hi, negative, scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn generates_valid_decimals(num in arb_decimal()) {
            prop_assert!(num.scale() <= Decimal::MAX_SCALE);
        }
    }
}
