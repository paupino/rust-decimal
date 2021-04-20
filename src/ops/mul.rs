use crate::decimal::{CalculationResult, Decimal, BIG_POWERS_10, MAX_PRECISION};
use crate::ops::common::MAX_I64_SCALE;

use num_traits::Zero;

pub(crate) fn mul_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
    let backup = d1.clone();
    let d1 = d1.unpack();
    let d2 = d2.unpack();
    let mut scale = d1.scale + d2.scale;

    // See if we can optimize this calculation depending on whether the hi bits are set
    if d1.hi | d1.mid == 0 {
        // Check if we're 32 bits on both sides.
        if d2.hi | d2.mid == 0 {
            let mut low64 = d1.lo as u64 * d2.lo as u64;
            if scale > MAX_PRECISION {
                // We've exceeded maximum scale so we need to start reducing the precision (aka
                // rounding) until we have something that fits.
                // If we're too big then we effectively round to zero.
                if scale > MAX_PRECISION + MAX_I64_SCALE {
                    return CalculationResult::Ok(Decimal::zero());
                }

                scale -= MAX_PRECISION + 1;
                let mut power = BIG_POWERS_10[scale as usize];

                let tmp = low64 / power;
                let remainder = low64 - tmp * power;
                low64 = tmp;

                // Round the result. Since the divisor was a power of 10, it's always even.
                power = power >> 1;
                if remainder >= power && (remainder > power || (low64 as u32 & 1) > 0) {
                    low64 += 1;
                }

                scale = MAX_PRECISION;
            }

            return CalculationResult::Ok(Decimal::from_parts(
                low64 as u32,
                (low64 >> 32) as u32,
                0,
                d2.negative ^ d1.negative,
                scale,
            ));
        }

        // We know that the left hand side is just 32 bits.
        let mut tmp = d1.lo as u64 * d2.lo as u64;
        let lo = tmp as u32;
        tmp = (d1.lo as u64 * d2.mid as u64).wrapping_add(tmp >> 32);
        let mut mid = tmp as u32;
        tmp = tmp >> 32;

        // Finally, depending on d2.hi determine if we scale before return
        let mut hi = tmp as u32;
        if d2.hi != 0 {
            tmp = tmp.wrapping_add(d1.lo as u64 * d2.hi as u64);
            if tmp > u32::MAX as u64 {
                mid = tmp as u32;
                hi = (tmp >> 32) as u32;
                // TODO: Skip scale
            } else {
                hi = tmp as u32;
            }
        }

        // TODO: check leading zeros, and skip scale
        return CalculationResult::Ok(Decimal::from_parts(lo, mid, hi, d1.negative ^ d2.negative, scale));
    } else if d2.mid | d2.hi == 0 {
        // TODO: Generated tests don't cover this yet.
        // We know that the right hand side is just 32 bits.
        let mut tmp = d2.lo as u64 * d1.lo as u64;
        let lo = tmp as u32;
        tmp = (d2.lo as u64 * d1.mid as u64).wrapping_add(tmp >> 32);
        let mut mid = tmp as u32;
        tmp = tmp >> 32;

        // Finally, depending on d2.hi determine if we scale before return
        let mut hi = tmp as u32;
        if d1.hi != 0 {
            tmp = tmp.wrapping_add(d2.lo as u64 * d1.hi as u64);
            if tmp > u32::MAX as u64 {
                mid = tmp as u32;
                hi = (tmp >> 32) as u32;
                // TODO: Skip scale
            } else {
                hi = tmp as u32;
            }
        }

        // TODO: check leading zeros, and skip scale
        return CalculationResult::Ok(Decimal::from_parts(lo, mid, hi, d1.negative ^ d2.negative, scale));
    } else {
    }
    unimplemented!("mul")
}
