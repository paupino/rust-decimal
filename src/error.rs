use crate::constants::{MAX_I128_REPR, MAX_PRECISION};
use alloc::string::String;
use core::fmt;

/// Error type for the library.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ErrorString(String),
    ExceedsMaximumPossibleValue(i128),
    LessThanMinimumPossibleValue(i128),
    ScaleExceedsMaximumPrecision(u32),
}

impl<S> From<S> for Error
where
    S: Into<String>,
{
    #[inline]
    fn from(from: S) -> Self {
        Self::ErrorString(from.into())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ErrorString(ref err) => f.pad(&err),
            Self::ExceedsMaximumPossibleValue(ref num) => write!(
                f,
                "Number exceeds maximum value that can be represented: {} > {}",
                num, MAX_I128_REPR
            ),
            Self::LessThanMinimumPossibleValue(ref num) => write!(
                f,
                "Number less than minimum value that can be represented: {} < {}",
                num, -MAX_I128_REPR
            ),
            Self::ScaleExceedsMaximumPrecision(ref scale) => {
                write!(f, "Scale exceeds maximum precision: {} > {}", scale, MAX_PRECISION)
            }
        }
    }
}
