use crate::{parse_radix_10_exact, DecimalComponents, ParserError, OVERFLOW_U96};
use std::ops::Rem;

#[inline]
fn assert_scale(scale: u32) -> Result<(), ParserError> {
    if scale > 28 {
        Err(ParserError::ScaleExceedsMaximumPrecision(scale))
    } else {
        Ok(())
    }
}

pub fn parse_scientific(input: &str) -> Result<DecimalComponents, ParserError> {
    let mut split = input.splitn(2, |c| c == 'e' || c == 'E');

    let base = split.next().ok_or_else(|| ParserError::UnableToExtractBase)?;
    let exp = split.next().ok_or_else(|| ParserError::UnableToExtractExponent)?;

    let mut decimal = parse_radix_10_exact(base)?;
    let mut mantissa = (decimal.lo as u128) | ((decimal.mid as u128) << 32) | ((decimal.hi as u128) << 64);

    if let Some(stripped) = exp.strip_prefix('-') {
        let exp: u32 = stripped.parse().map_err(|_| ParserError::UnableToParseExponent)?;
        decimal.scale += exp;
        assert_scale(decimal.scale)?;
    } else {
        let exp: u32 = exp.parse().map_err(|_| ParserError::UnableToParseExponent)?;
        if exp <= decimal.scale {
            decimal.scale -= exp;
            assert_scale(decimal.scale)?;
        } else if exp > 0 {
            // This is a case whereby the mantissa needs to be larger to be correctly
            // represented within the decimal type. A good example is 1.2E10. At this point,
            // we've parsed 1.2 as the base and 10 as the exponent. To represent this within a
            // Decimal type we effectively store the mantissa as 12,000,000,000 and scale as
            // zero.
            assert_scale(exp)?;

            let pow = 10_u128.pow(exp);
            mantissa = match mantissa.checked_mul(pow) {
                Some(m) => m,
                None => return Err(ParserError::ExceedsMaximumPossibleValue),
            };
            if mantissa >= OVERFLOW_U96 {
                return Err(ParserError::ExceedsMaximumPossibleValue);
            }
        }
    }

    // Lastly, remove any trailing zeros. This is unique to scientific parsing.
    while decimal.scale > 0 {
        let remainder = mantissa.rem(10);
        if remainder != 0 {
            break;
        }
        mantissa /= 10;
        decimal.scale -= 1;
    }

    decimal.lo = mantissa as u32;
    decimal.mid = (mantissa >> 32) as u32;
    decimal.hi = (mantissa >> 64) as u32;
    Ok(decimal)
}
