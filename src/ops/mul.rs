use crate::decimal::{CalculationResult, Decimal, BIG_POWERS_10, MAX_PRECISION};
use crate::ops::common::{Buf24, MAX_I64_SCALE};

use num_traits::Zero;

pub(crate) fn mul_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
    let d1 = d1.unpack();
    let d2 = d2.unpack();
    let mut scale = d1.scale + d2.scale;
    let negative = d1.negative ^ d2.negative;
    let mut product = Buf24::zero();

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

            // Early exit
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
        product.u0 = tmp as u32;
        tmp = (d1.lo as u64 * d2.mid as u64).wrapping_add(tmp >> 32);
        product.u1 = tmp as u32;
        tmp = tmp >> 32;

        // Finally, depending on d2.hi determine if we scale before return
        if d2.hi != 0 {
            tmp = tmp.wrapping_add(d1.lo as u64 * d2.hi as u64);
            if tmp > u32::MAX as u64 {
                product.set_mid64(tmp);
            } else {
                product.u2 = tmp as u32;
            }
        } else {
            product.u2 = tmp as u32;
        }
    } else if d2.mid | d2.hi == 0 {
        // We know that the right hand side is just 32 bits.
        let mut tmp = d2.lo as u64 * d1.lo as u64;
        product.u0 = tmp as u32;
        tmp = (d2.lo as u64 * d1.mid as u64).wrapping_add(tmp >> 32);
        product.u1 = tmp as u32;
        tmp = tmp >> 32;

        // Finally, depending on d2.hi determine if we scale before return
        if d1.hi != 0 {
            tmp = tmp.wrapping_add(d2.lo as u64 * d1.hi as u64);
            if tmp > u32::MAX as u64 {
                product.set_mid64(tmp);
            } else {
                product.u2 = tmp as u32;
            }
        } else {
            product.u2 = tmp as u32;
        }
    } else {
        // We're not dealing with 32 bit numbers on either side. Both operands are > 32 bits.
        // We compute and accumulate the 9 partial products using long multiplication
        let mut tmp = d1.lo as u64 * d2.lo as u64; // 1
        product.u0 = tmp as u32;
        let mut tmp2 = (d1.lo as u64 * d2.mid as u64).wrapping_add(tmp >> 32); // 2
        tmp = d1.mid as u64 * d2.lo as u64; // 3
        tmp = tmp.wrapping_add(tmp2);
        product.u1 = tmp as u32;

        // Detect if carry happened from the wrapping add
        if tmp < tmp2 {
            tmp2 = (tmp >> 32) | (1u64 << 32);
        } else {
            tmp2 = tmp >> 32;
        }

        tmp = (d1.mid as u64 * d2.mid as u64) + tmp2; // 4
        if (d1.hi | d2.hi) > 0 {
            // We need to calculate 5 more partial products.
            tmp2 = d1.lo as u64 * d2.hi as u64; // 5
            tmp = tmp.wrapping_add(tmp2);

            // Detect if wrapping add carried
            let mut tmp3 = if tmp < tmp2 { 1 } else { 0 };
            tmp2 = d1.hi as u64 * d2.lo as u64; // 6
            tmp = tmp.wrapping_add(tmp2);
            product.u2 = tmp as u32;
            // Detect if wrapping add carried
            if tmp < tmp2 {
                tmp3 += 1;
            }
            tmp2 = (tmp3 << 32) | (tmp >> 32);

            tmp = d1.mid as u64 * d2.hi as u64; // 7
            tmp = tmp.wrapping_add(tmp2);
            // Detect if wrapping add carried
            tmp3 = if tmp < tmp2 { 1 } else { 0 };

            tmp2 = d1.hi as u64 * d2.mid as u64; // 8
            tmp = tmp.wrapping_add(tmp2);
            product.u3 = tmp as u32;
            if tmp < tmp2 {
                tmp3 += 1;
            }
            tmp = (tmp3 << 32) | (tmp >> 32);

            product.set_high64(d1.hi as u64 * d2.hi as u64 + tmp);
        } else {
            product.set_mid64(tmp);
        }
    }
    // Scale as necessary
    let upper_word = product.upper_word();
    if upper_word > 2 || scale > MAX_PRECISION {
        scale = if let Some(new_scale) = product.rescale(upper_word, scale) {
            new_scale
        } else {
            return CalculationResult::Overflow;
        }
    }

    CalculationResult::Ok(Decimal::from_parts(product.u0, product.u1, product.u2, negative, scale))
}
