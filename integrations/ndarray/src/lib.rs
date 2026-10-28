//! [`ndarray`] scalar support for [`rust_decimal::Decimal`].
//!
//! Wrap the scalar in [`ScalarDecimal`] to use it with array arithmetic:
//!
//! ```
//! use ndarray::array;
//! use rust_decimal::Decimal;
//! use rust_decimal_ndarray::ScalarDecimal;
//!
//! let a = array![Decimal::ONE, Decimal::TWO];
//! assert_eq!(a * ScalarDecimal(Decimal::TEN), array![Decimal::TEN, Decimal::from(20)]);
//! ```
#![no_std]
#![forbid(unsafe_code)]

use core::ops::{Add, Div, Mul, Rem, Sub};
use rust_decimal::Decimal;

/// A [`Decimal`] usable as an [`ndarray::ScalarOperand`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScalarDecimal(pub Decimal);

impl ndarray::ScalarOperand for ScalarDecimal {}

impl From<Decimal> for ScalarDecimal {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<ScalarDecimal> for Decimal {
    fn from(value: ScalarDecimal) -> Self {
        value.0
    }
}

macro_rules! scalar_op {
    ($($trait:ident $method:ident),*) => {$(
        impl $trait<ScalarDecimal> for Decimal {
            type Output = Decimal;
            fn $method(self, rhs: ScalarDecimal) -> Decimal {
                $trait::$method(self, rhs.0)
            }
        }
    )*};
}

scalar_op!(Add add, Sub sub, Mul mul, Div div, Rem rem);
