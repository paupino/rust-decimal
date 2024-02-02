mod base_10;
mod scientific;

pub use base_10::{parse_radix_10, parse_radix_10_exact};
pub use scientific::parse_scientific;

// Determines potential overflow for 128 bit operations
pub(crate) const OVERFLOW_U96: u128 = 1u128 << 96;
pub(crate) const BYTES_TO_OVERFLOW_U64: usize = 18; // We can probably get away with less
pub(crate) const WILL_OVERFLOW_U64: u64 = u64::MAX / 10 - u8::MAX as u64;

pub enum ParserError {
    /// Parser input was empty
    EmptyInput,
    /// Exceeds maximum value for a Decimal
    ExceedsMaximumPossibleValue,
    /// Placeholder was present in an invalid position
    InvalidPlaceholderPosition,
    /// Invalid character was found when parsing
    InvalidCharacter,
    /// Multiple decimal points were found in the input string
    MultipleDecimalPoints,
    /// Number contained valid characters, none of which were digits
    NoDigits,
    /// Number overflowed
    Overflow,
    /// Number overflowed after rounding was attempted
    OverflowAfterRound,
    /// Number overflowed when carrying
    OverflowCarry,
    /// Number overflowed when reducing scale
    OverflowScale,
    /// Scale is larger than the maximum precision
    ScaleExceedsMaximumPrecision(u32),
    /// When parsing scientific notation, the base could not be extracted
    UnableToExtractBase,
    /// When parsing scientific notation, the exponent could not be extracted
    UnableToExtractExponent,
    /// When parsing scientific notation, the exponent could not be parsed
    UnableToParseExponent,
    /// Number underflowed
    Underflow,
    // The radix provided is unsupported (i.e. < 2 or > 36)
    UnsupportedRadix(u32),
}

impl core::fmt::Display for ParserError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParserError::EmptyInput => write!(f, "empty"),
            ParserError::ExceedsMaximumPossibleValue => {
                write!(f, "number exceeds maximum value that can be represented")
            }
            ParserError::InvalidPlaceholderPosition => write!(f, "must start lead with a number"),
            ParserError::InvalidCharacter => write!(f, "unknown character"),
            ParserError::MultipleDecimalPoints => write!(f, "two decimal points"),
            ParserError::NoDigits => write!(f, "no digits found"),
            ParserError::Overflow => write!(f, "overflow from too many digits"),
            ParserError::OverflowAfterRound => write!(f, "overflow from mantissa after rounding"),
            ParserError::OverflowCarry => write!(f, "overflow from carry"),
            ParserError::OverflowScale => write!(f, "overflow from scale mismatch"),
            ParserError::ScaleExceedsMaximumPrecision(scale) => {
                write!(f, "scale exceeds the maximum representable precision: {}", scale)
            }
            ParserError::UnableToExtractBase => write!(f, "failed to parse"),
            ParserError::UnableToExtractExponent => write!(f, "failed to parse"),
            ParserError::UnableToParseExponent => write!(f, "failed to parse"),
            ParserError::Underflow => write!(f, "number has a high precision that can not be represented"),
            ParserError::UnsupportedRadix(radix) if *radix < 2 => write!(f, "unsupported radix < 2"),
            ParserError::UnsupportedRadix(radix) if *radix > 36 => write!(f, "unsupported radix > 36"),
            ParserError::UnsupportedRadix(_) => write!(f, "unsupported radix"),
        }
    }
}

#[derive(Debug)]
pub struct DecimalComponents {
    pub lo: u32,
    pub mid: u32,
    pub hi: u32,
    pub negative: bool,
    pub scale: u32,
}
