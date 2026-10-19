use crate::constants::{BIG_POWERS_10, MAX_I64_SCALE, POWERS_10};
use crate::decimal::{CalculationResult, Decimal};
use crate::ops::common::Buf24;

#[inline(always)]
pub(crate) fn mul_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
    let scale = d1.scale() + d2.scale();
    let negative = d1.is_sign_negative() ^ d2.is_sign_negative();

    if d1.mid() | d1.hi() | d2.mid() | d2.hi() == 0 {
        // We're multiplying two 32 bit integers (the most common case), so the product fits
        // within 64 bits.
        let product = d1.lo() as u64 * d2.lo() as u64;
        if product == 0 {
            // We should think about this - does zero need to maintain precision? This treats it like
            // an absolute which I think is ok, especially since we have is_zero() functions etc.
            return CalculationResult::Ok(Decimal::ZERO);
        }
        if scale > Decimal::MAX_SCALE {
            return CalculationResult::Ok(mul_32_rescale(product, negative, scale));
        }
        return CalculationResult::Ok(Decimal::from_parts_raw_unchecked(
            product as u32,
            (product >> 32) as u32,
            0,
            crate::decimal::flags(negative, scale),
        ));
    }

    if d1.is_zero() || d2.is_zero() {
        return CalculationResult::Ok(Decimal::ZERO);
    }

    if d1.hi() | d2.hi() == 0 {
        // Both operands fit within 64 bits, so the full product fits within 128 bits and can be
        // computed with a single widening multiply.
        let product = low64(d1) as u128 * low64(d2) as u128;
        if scale > Decimal::MAX_SCALE && product >> 64 == 0 {
            return CalculationResult::Ok(rescale_u64(product as u64, negative, scale));
        }
        return finish_mul(product, 0, negative, scale);
    }

    // At least one operand is 96 bits. Split each mantissa into a low 64 bit and a high 32 bit part
    // and accumulate the 192 bit product from three widening multiplies and one 32x32 multiply.
    let (a_lo, a_hi) = (low64(d1), d1.hi() as u64);
    let (b_lo, b_hi) = (low64(d2), d2.hi() as u64);
    let low = a_lo as u128 * b_lo as u128;
    // Each cross product is below 2^96, so the sum can't overflow.
    let cross = a_lo as u128 * b_hi as u128 + a_hi as u128 * b_lo as u128;
    let mid = (low >> 64) + (cross as u64 as u128);
    let low128 = (low as u64 as u128) | (mid << 64);
    let high64 = a_hi * b_hi + (cross >> 64) as u64 + (mid >> 64) as u64;
    finish_mul(low128, high64, negative, scale)
}

#[inline(always)]
const fn low64(d: &Decimal) -> u64 {
    ((d.mid() as u64) << 32) | d.lo() as u64
}

/// Rounds the non-zero product of two 32 bit mantissas whose combined scale exceeds the maximum.
#[inline(always)]
fn mul_32_rescale(product: u64, negative: bool, scale: u32) -> Decimal {
    // We've exceeded maximum scale so we need to start reducing the precision (aka
    // rounding) until we have something that fits.
    // If we're too big then we effectively round to zero.
    if scale > Decimal::MAX_SCALE + MAX_I64_SCALE {
        return Decimal::ZERO;
    }
    let quotient = div_round_pow10(product, scale - Decimal::MAX_SCALE);
    // Unlike the other paths, this drops the sign if the result rounds to zero.
    Decimal::from_parts(
        quotient as u32,
        (quotient >> 32) as u32,
        0,
        negative,
        Decimal::MAX_SCALE,
    )
}

/// Rounds a non-zero product that fits within 64 bits but whose scale exceeds the maximum. This
/// gives the same result as `rescale_product`, but a single hardware division is cheaper.
#[inline(never)]
fn rescale_u64(product: u64, negative: bool, scale: u32) -> Decimal {
    let power = scale - Decimal::MAX_SCALE;
    // Any 64 bit value rounds to zero when dropping 20 or more digits.
    let quotient = if power > MAX_I64_SCALE {
        0
    } else {
        div_round_pow10(product, power)
    };
    Decimal::from_parts_raw_unchecked(
        quotient as u32,
        (quotient >> 32) as u32,
        0,
        crate::decimal::flags(negative, Decimal::MAX_SCALE),
    )
}

/// Divides `value` by 10^power (1 <= power <= 19), rounding half to even.
#[inline(always)]
fn div_round_pow10(value: u64, power: u32) -> u64 {
    let divisor = BIG_POWERS_10[power as usize - 1];
    let quotient = value / divisor;
    let remainder = value - quotient * divisor;
    quotient + rounds_up(remainder, divisor >> 1, quotient & 1 != 0) as u64
}

/// Converts the 192 bit product `high64:low128` at the given (combined) scale into a Decimal,
/// rescaling it if the mantissa exceeds 96 bits or the scale exceeds the maximum.
#[inline(always)]
fn finish_mul(low128: u128, high64: u64, negative: bool, scale: u32) -> CalculationResult {
    if high64 != 0 || low128 >> 96 != 0 || scale > Decimal::MAX_SCALE {
        let rescaled = rescale_product(low128, high64, negative, scale);
        if rescaled == RESCALE_OVERFLOW {
            return CalculationResult::Overflow;
        }
        return CalculationResult::Ok(Decimal::from_parts_raw_unchecked(
            rescaled as u32,
            (rescaled >> 32) as u32,
            (rescaled >> 64) as u32,
            (rescaled >> 96) as u32,
        ));
    }

    // Result is non-zero (both inputs are non-zero, checked at entry of mul_impl)
    CalculationResult::Ok(Decimal::from_parts_raw_unchecked(
        low128 as u32,
        (low128 >> 32) as u32,
        (low128 >> 64) as u32,
        crate::decimal::flags(negative, scale),
    ))
}

/// Rescales a non-zero 192 bit product `high64:low128` at `scale` so that the mantissa fits within
/// 96 bits and the scale is at most `MAX_SCALE`, rounding half to even.
///
/// This produces the same results as the general purpose `Buf24::rescale`: the fewest digits
/// needed are dropped, rounding takes every dropped digit into account, and if rounding up carries
/// out of 96 bits one more digit is dropped. The result may round to zero, in which case the sign
/// is kept.
///
/// Returns the new mantissa in the low 96 bits and the new flags in the upper 32 bits, or
/// `RESCALE_OVERFLOW` if the product can't be represented. Packing the result into a u128 lets it
/// be returned in registers.
#[inline(never)]
fn rescale_product(low128: u128, high64: u64, negative: bool, scale: u32) -> u128 {
    // The number of digits to drop so that the quotient fits within 96 bits is the number of
    // digits in `product / 2^96`, since `product < 2^96 * 10^k` iff `product / 2^96 < 10^k`.
    let mut power = if high64 == 0 {
        // The product fits within 128 bits (the common case), so the top part fits within 32 bits.
        let top = (low128 >> 96) as u32;
        if top == 0 { 0 } else { decimal_digits_u32(top) }
    } else {
        decimal_digits_u128(((high64 as u128) << 32) | (low128 >> 96))
    };
    // We also need to drop enough digits to bring the scale into the valid range.
    if power + Decimal::MAX_SCALE < scale {
        power = scale - Decimal::MAX_SCALE;
    }
    if power > scale {
        return RESCALE_OVERFLOW;
    }
    if power as usize > MAX_POW10_U64 + POW10_DIVISORS_128.len() {
        return rescale_product_wide(low128, high64, negative, scale);
    }
    let mut scale = scale - power;

    // Divide by 10^power. Since `product < 2^96 * 10^power`, the quotient fits within 96 bits.
    // The remainder and divisor are both shifted left by the same amount, so the remainder can be
    // compared with half of the shifted divisor.
    let dividend = [low128 as u64, (low128 >> 64) as u64, high64];
    let (mut quotient, round_up) = if power <= MAX_POW10_U64 as u32 {
        let divisor = &POW10_DIVISORS[power as usize];
        let (quotient, remainder) = divisor.div_rem_to_128(dividend);
        (quotient, rounds_up(remainder, divisor.divisor >> 1, quotient & 1 != 0))
    } else {
        let divisor = &POW10_DIVISORS_128[power as usize - MAX_POW10_U64 - 1];
        let (quotient, remainder) = divisor.div_rem_to_128(dividend);
        (quotient, rounds_up(remainder, divisor.divisor >> 1, quotient & 1 != 0))
    };

    if round_up {
        quotient += 1;
        if quotient >> 96 != 0 {
            // Rounding carried out of 96 bits, so the quotient is exactly 2^96 and we need to drop
            // one more digit. 2^96 / 10 has a remainder of 6, so this always rounds up.
            if scale == 0 {
                return RESCALE_OVERFLOW;
            }
            scale -= 1;
            quotient = (1u128 << 96) / 10 + 1;
        }
    }

    quotient | ((crate::decimal::flags(negative, scale) as u128) << 96)
}

/// Rescales a product that needs more than 29 digits dropped, which only happens when an operand
/// has a scale above the maximum (possible through `bytemuck`). This is beyond the reciprocal tables,
/// so it uses the general purpose `Buf24::rescale`.
#[cold]
#[inline(never)]
fn rescale_product_wide(low128: u128, high64: u64, negative: bool, scale: u32) -> u128 {
    let mut product = Buf24::zero();
    product.set_low64(low128 as u64);
    product.set_mid64((low128 >> 64) as u64);
    product.set_high64(high64);
    let Some(scale) = product.rescale(product.upper_word(), scale) else {
        return RESCALE_OVERFLOW;
    };
    let mantissa = product.data[0] as u128 | ((product.data[1] as u128) << 32) | ((product.data[2] as u128) << 64);
    mantissa | ((crate::decimal::flags(negative, scale) as u128) << 96)
}

/// Whether a quotient should be rounded up given its remainder, using round half to even.
#[inline(always)]
fn rounds_up<T: PartialOrd>(remainder: T, half: T, odd: bool) -> bool {
    remainder > half || (remainder == half && odd)
}

/// Returned by `rescale_product` when the product can't be represented. It can't be confused with
/// a valid result since its flags hold a scale above the maximum.
const RESCALE_OVERFLOW: u128 = u128::MAX;

// The number of decimal digits in a non-zero value is estimated from its bit length (1233 / 4096
// ~= log10(2)). The estimate is either exact or one too low, which a comparison corrects.

/// Returns the number of decimal digits in `value`, which must be non-zero.
#[inline(always)]
fn decimal_digits_u32(value: u32) -> u32 {
    let estimate = ((32 - value.leading_zeros()) * 1233) >> 12;
    estimate + (value >= POWERS_10[estimate as usize]) as u32
}

/// Returns the number of decimal digits in `value`, which must be non-zero.
#[inline(always)]
fn decimal_digits_u128(value: u128) -> u32 {
    let estimate = ((128 - value.leading_zeros()) * 1233) >> 12;
    estimate + (value >= POW10_U128[estimate as usize]) as u32
}

const POW10_U128: [u128; 39] = {
    let mut table = [1u128; 39];
    let mut i = 1;
    while i < table.len() {
        table[i] = table[i - 1] * 10;
        i += 1;
    }
    table
};

/// The largest power of 10 that fits within a u64.
const MAX_POW10_U64: usize = MAX_I64_SCALE as usize;

/// 10^0 to 10^19 as divisors.
const POW10_DIVISORS: [NormalizedDivisor; MAX_POW10_U64 + 1] = {
    let mut table = [NormalizedDivisor::new(1); MAX_POW10_U64 + 1];
    let mut power = 1;
    let mut i = 1;
    while i <= MAX_POW10_U64 {
        power *= 10;
        table[i] = NormalizedDivisor::new(power);
        i += 1;
    }
    table
};

/// A 64 bit divisor shifted left so that its top bit is set, along with its reciprocal as defined
/// by Möller and Granlund, "Improved division by invariant integers" (2011). This lets us divide a
/// 128 bit value by the divisor using multiplication rather than a hardware division.
#[derive(Clone, Copy)]
struct NormalizedDivisor {
    divisor: u64,
    reciprocal: u64,
    shift: u32,
}

impl NormalizedDivisor {
    const fn new(d: u64) -> Self {
        let shift = d.leading_zeros();
        let divisor = d << shift;
        // floor((2^128 - 1) / divisor) - 2^64 fits within 64 bits since the divisor is normalized.
        let reciprocal = (u128::MAX / divisor as u128 - (1u128 << 64)) as u64;
        NormalizedDivisor {
            divisor,
            reciprocal,
            shift,
        }
    }

    /// Divides `u1:u0` by the normalized divisor, returning the quotient and remainder. Requires
    /// `u1 < divisor`, which guarantees the quotient fits within 64 bits.
    #[inline(always)]
    const fn div_rem_2by1(&self, u1: u64, u0: u64) -> (u64, u64) {
        let d = self.divisor;
        let q = (self.reciprocal as u128 * u1 as u128).wrapping_add(((u1 as u128) << 64) | u0 as u128);
        let mut q1 = ((q >> 64) as u64).wrapping_add(1);
        let q0 = q as u64;
        let mut r = u0.wrapping_sub(q1.wrapping_mul(d));
        if r > q0 {
            q1 = q1.wrapping_sub(1);
            r = r.wrapping_add(d);
        }
        if r >= d {
            q1 += 1;
            r -= d;
        }
        (q1, r)
    }

    /// Divides the 192 bit little endian `value` by the original (unshifted) divisor, where the
    /// quotient is known to fit within 128 bits. Returns the quotient and the remainder shifted
    /// left by `self.shift`.
    #[inline(always)]
    const fn div_rem_to_128(&self, value: [u64; 3]) -> (u128, u64) {
        // Shift the dividend by the same amount as the divisor. Since the quotient fits within 128
        // bits, the shifted dividend fits within 192 bits and its top word is below the divisor.
        // `>> 1 >> (63 - shift)` avoids an overflowing shift when `shift` is zero.
        let shift = self.shift;
        let [x0, x1, x2] = value;
        let n0 = x0 << shift;
        let n1 = (x1 << shift) | (x0 >> 1 >> (63 - shift));
        let n2 = (x2 << shift) | (x1 >> 1 >> (63 - shift));
        let (q1, r) = self.div_rem_2by1(n2, n1);
        let (q0, r) = self.div_rem_2by1(r, n0);
        (((q1 as u128) << 64) | q0 as u128, r)
    }
}

/// 10^20 to 10^29 as divisors.
const POW10_DIVISORS_128: [NormalizedDivisor128; 10] = {
    let mut table = [NormalizedDivisor128::new(1 << 127); 10];
    let mut power = 10u128.pow(MAX_POW10_U64 as u32);
    let mut i = 0;
    while i < table.len() {
        power *= 10;
        table[i] = NormalizedDivisor128::new(power);
        i += 1;
    }
    table
};

/// A 128 bit divisor shifted left so that its top bit is set, along with its reciprocal as defined
/// by Möller and Granlund, "Improved division by invariant integers" (2011).
#[derive(Clone, Copy)]
struct NormalizedDivisor128 {
    divisor: u128,
    reciprocal: u64,
    shift: u32,
}

impl NormalizedDivisor128 {
    const fn new(d: u128) -> Self {
        let shift = d.leading_zeros();
        let divisor = d << shift;
        // floor((2^192 - 1) / divisor) - 2^64, computed with binary long division. The quotient is
        // in [2^64, 2^65) since the divisor is normalized.
        let mut quotient = 0u128;
        let mut remainder = 0u128;
        let mut i = 0;
        while i < 192 {
            let carry = remainder >> 127;
            remainder = (remainder << 1) | 1;
            quotient <<= 1;
            if carry != 0 || remainder >= divisor {
                remainder = remainder.wrapping_sub(divisor);
                quotient |= 1;
            }
            i += 1;
        }
        NormalizedDivisor128 {
            divisor,
            reciprocal: (quotient - (1 << 64)) as u64,
            shift,
        }
    }

    /// Divides `u2:u1:u0` by the normalized divisor, returning the quotient and remainder. Requires
    /// `u2:u1 < divisor`, which guarantees the quotient fits within 64 bits.
    #[inline(always)]
    const fn div_rem_3by2(&self, u2: u64, u1: u64, u0: u64) -> (u64, u128) {
        let d = self.divisor;
        let d1 = (d >> 64) as u64;
        let d0 = d as u64;
        let q = (self.reciprocal as u128 * u2 as u128).wrapping_add(((u2 as u128) << 64) | u1 as u128);
        let mut q1 = (q >> 64) as u64;
        let q0 = q as u64;
        let r1 = u1.wrapping_sub(q1.wrapping_mul(d1));
        let t = d0 as u128 * q1 as u128;
        let mut r = (((r1 as u128) << 64) | u0 as u128).wrapping_sub(t).wrapping_sub(d);
        q1 = q1.wrapping_add(1);
        if (r >> 64) as u64 >= q0 {
            q1 = q1.wrapping_sub(1);
            r = r.wrapping_add(d);
        }
        if r >= d {
            q1 += 1;
            r -= d;
        }
        (q1, r)
    }

    /// Divides the 192 bit little endian `value` by the original (unshifted) divisor, where the
    /// quotient is known to fit within 128 bits. Returns the quotient and the remainder shifted
    /// left by `self.shift`.
    #[inline(always)]
    const fn div_rem_to_128(&self, value: [u64; 3]) -> (u128, u128) {
        // Shift the dividend by the same amount as the divisor (always less than 64 bits for the
        // powers of 10 we use), giving four words. Since the quotient fits within 128 bits, the top
        // two words are below the divisor.
        let shift = self.shift;
        let [x0, x1, x2] = value;
        let n0 = x0 << shift;
        let n1 = (x1 << shift) | (x0 >> 1 >> (63 - shift));
        let n2 = (x2 << shift) | (x1 >> 1 >> (63 - shift));
        let n3 = x2 >> 1 >> (63 - shift);
        let (q1, r) = self.div_rem_3by2(n3, n2, n1);
        let (q0, r) = self.div_rem_3by2((r >> 64) as u64, r as u64, n0);
        (((q1 as u128) << 64) | q0 as u128, r)
    }
}

#[cfg(test)]
mod tests;
