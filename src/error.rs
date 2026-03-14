use crate::Decimal;
use core::fmt;

/// Error type for the library.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Represents a failure to convert to/from `Decimal` to the specified type. This is typically
    /// due to type constraints (e.g. `Decimal::MAX` cannot be converted into `i32`).
    ConversionTo(&'static str),
    /// The decimal string contained more than one decimal point.
    DuplicatedDecimalPoint,
    /// Could not represent a Decimal instance because there no data left.
    EmptyData,
    /// The value provided exceeds `Decimal::MAX`.
    ExceedsMaximumPossibleValue,
    /// A string could not represent a scientific number.
    FailedToParseScientificFromString,
    /// A character could not represent a Decimal instance
    InvalidCharacter,
    /// The string must start with a digit, `+` or `-`.
    InvalidLeadingChar,
    /// The value provided is less than `Decimal::MIN`.
    LessThanMinimumPossibleValue,
    /// The string did not contain any digits.
    NoDigits,
    /// The scale provided exceeds the maximum scale that `Decimal` can represent.
    ScaleExceedsMaximumPrecision(u32),
    /// An underflow is when there are more fractional digits than can be represented within `Decimal`.
    Underflow,
    /// The radix is not supported. Must be between 2 and 36.
    UnsupportedRadix,
}

#[cold]
pub(crate) fn tail_error(from: Error) -> Result<Decimal, Error> {
    Err(from)
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ConversionTo(ref type_name) => {
                write!(f, "Error while converting to {type_name}")
            }
            Self::DuplicatedDecimalPoint => {
                write!(f, "The decimal string contained more than one decimal point.")
            }
            Self::EmptyData => {
                write!(f, "Could not represent a Decimal instance because there no data left.")
            }
            Self::ExceedsMaximumPossibleValue => {
                write!(f, "Number exceeds maximum value that can be represented.")
            }
            Self::FailedToParseScientificFromString => {
                write!(f, "A string could not represent a scientific number.")
            }
            Self::InvalidCharacter => {
                write!(f, "A character could not represent a Decimal instance.")
            }
            Self::InvalidLeadingChar => {
                write!(f, "The string must start with a digit, `+` or `-`.")
            }
            Self::LessThanMinimumPossibleValue => {
                write!(f, "Number less than minimum value that can be represented.")
            }
            Self::NoDigits => {
                write!(f, "The string did not contain any digits.")
            }
            Self::ScaleExceedsMaximumPrecision(ref scale) => {
                write!(
                    f,
                    "Scale exceeds the maximum precision allowed: {scale} > {}",
                    Decimal::MAX_SCALE
                )
            }
            Self::Underflow => {
                write!(f, "Number has a high precision that can not be represented.")
            }
            Self::UnsupportedRadix => {
                write!(f, "The radix is not supported. Must be between 2 and 36.")
            }
        }
    }
}
