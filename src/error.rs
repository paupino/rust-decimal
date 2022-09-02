#[cfg(doc)]
use crate::Decimal;
use core::fmt;

/// An error which can be returned when parsing [`Decimal`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseDecimalError {
    /// Value being parsed is empty.
    Empty,
    /// Contains an invalid digit in its context.
    InvalidDigit,
    /// Number is too large to fit in a `Decimal`.
    PosOverflow,
    /// Number is too small to fit in a `Decimal`.
    NegOverflow,
    /// Number has too many digits to fit in a `Decimal`.
    Underflow,
    #[doc(hidden)]
    __Internal,
    #[doc(hidden)]
    __Generic,
}

/// The error type returned when checked type conversion from [`Decimal`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryFromDecimalError {
    pub(crate) _priv: (),
}

/// The error type returned when checked type conversion into [`Decimal`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryIntoDecimalError {
    pub(crate) _priv: (),
}

#[cfg(feature = "std")]
impl std::error::Error for ParseDecimalError {}

#[cfg(feature = "std")]
impl std::error::Error for TryFromDecimalError {}

#[cfg(feature = "std")]
impl std::error::Error for TryIntoDecimalError {}

impl fmt::Display for ParseDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseDecimalError::Empty => "cannot parse decimal from empty string".fmt(f),
            ParseDecimalError::InvalidDigit => "invalid digit found in string".fmt(f),
            ParseDecimalError::PosOverflow => "number is too large to fit in a decimal".fmt(f),
            ParseDecimalError::NegOverflow => "number is too small to fit in a decimal".fmt(f),
            ParseDecimalError::Underflow => "number has too many digits to fit in a decimal".fmt(f),
            ParseDecimalError::__Internal => "rust_decimal encountered an unexpected error condition".fmt(f),
            ParseDecimalError::__Generic => "failed to parse".fmt(f),
        }
    }
}

impl fmt::Display for TryFromDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "lossy conversion from decimal attempted".fmt(f)
    }
}

impl fmt::Display for TryIntoDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "lossy conversion into decimal attempted".fmt(f)
    }
}
