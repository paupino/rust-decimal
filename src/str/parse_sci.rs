use crate::{constants::MAX_PRECISION_U32, error::Error, Decimal};

use core::str::FromStr;

#[inline]
pub(crate) fn parse_str_scientific(value: &str) -> Result<Decimal, crate::Error> {
    const ERROR_MESSAGE: &str = "Failed to parse";

    let mut split = value.splitn(2, |c| c == 'e' || c == 'E');

    let base = split.next().ok_or_else(|| Error::from(ERROR_MESSAGE))?;
    let exp = split.next().ok_or_else(|| Error::from(ERROR_MESSAGE))?;

    let mut ret = Decimal::from_str(base)?;
    let current_scale = ret.scale();

    if let Some(stripped) = exp.strip_prefix('-') {
        let exp: u32 = stripped.parse().map_err(|_| Error::from(ERROR_MESSAGE))?;
        ret.set_scale(current_scale + exp)?;
    } else {
        let exp: u32 = exp.parse().map_err(|_| Error::from(ERROR_MESSAGE))?;
        if exp <= current_scale {
            ret.set_scale(current_scale - exp)?;
        } else if exp > 0 {
            use crate::constants::BIG_POWERS_10;

            // This is a case whereby the mantissa needs to be larger to be correctly
            // represented within the decimal type. A good example is 1.2E10. At this point,
            // we've parsed 1.2 as the base and 10 as the exponent. To represent this within a
            // Decimal type we effectively store the mantissa as 12,000,000,000 and scale as
            // zero.
            if exp > MAX_PRECISION_U32 {
                return Err(Error::ScaleExceedsMaximumPrecision(exp));
            }
            let mut exp = exp as usize;
            // Max two iterations. If exp is 1 then it needs to index position 0 of the array.
            while exp > 0 {
                let pow;
                if exp >= BIG_POWERS_10.len() {
                    pow = BIG_POWERS_10[BIG_POWERS_10.len() - 1];
                    exp -= BIG_POWERS_10.len();
                } else {
                    pow = BIG_POWERS_10[exp - 1];
                    exp = 0;
                }

                let pow = Decimal::from_parts_raw(pow as u32, (pow >> 32) as u32, 0, 0);
                match ret.checked_mul(pow) {
                    Some(r) => ret = r,
                    None => return Err(Error::ExceedsMaximumPossibleValue),
                };
            }
            ret.normalize_assign();
        }
    }
    Ok(ret)
}
