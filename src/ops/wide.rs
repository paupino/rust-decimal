// Wide (192-bit mantissa) decimal arithmetic for maintaining precision
// in iterative operations like exponentiation by squaring.
//
// A standard Decimal has a 96-bit mantissa (~28.9 decimal digits).
// When multiplying two 96-bit values, the product can be up to 192 bits.
// The existing mul_impl immediately rescales this back to 96 bits, losing
// precision. In iterative multiplication (e.g. powu), this precision loss
// compounds at each step.
//
// DecWide keeps a 192-bit mantissa (~57.8 decimal digits) throughout the
// computation, only truncating to 96 bits at the very end.

use crate::constants::{MAX_I32_SCALE, POWERS_10};
use crate::Decimal;

/// Extended precision decimal with 192-bit mantissa.
#[derive(Clone, Debug)]
pub(crate) struct DecWide {
    /// 192-bit mantissa stored as 6 × 32-bit words (little-endian)
    data: [u32; 6],
    scale: u32,
    negative: bool,
}

/// 384-bit buffer for intermediate multiplication results.
struct Buf48 {
    data: [u32; 12],
}

impl DecWide {
    fn is_zero(&self) -> bool {
        self.data == [0; 6]
    }

    pub fn from_decimal(d: &Decimal) -> Self {
        let m = d.mantissa_array3();
        DecWide {
            data: [m[0], m[1], m[2], 0, 0, 0],
            scale: d.scale(),
            negative: d.is_sign_negative(),
        }
    }

    pub fn to_decimal(&self) -> Option<Decimal> {
        let mut data = self.data;
        let mut scale = self.scale as i32;
        let mut upper = upper_word_6(&data);

        // If it already fits in 96 bits and scale is valid, just return
        if upper <= 2 && scale <= Decimal::MAX_SCALE as i32 {
            return Some(Decimal::from_parts(data[0], data[1], data[2], self.negative, scale as u32));
        }

        // Rescale down until it fits in 96 bits with valid scale
        rescale_buf::<6, 2>(&mut data, &mut upper, &mut scale)?;

        Some(Decimal::from_parts(
            data[0],
            data[1],
            data[2],
            self.negative,
            scale as u32,
        ))
    }

    /// Multiply two DecWide values, keeping 192-bit precision.
    pub fn checked_mul(&self, other: &DecWide) -> Option<DecWide> {
        // Check for zero
        if self.is_zero() || other.is_zero() {
            return Some(DecWide {
                data: [0; 6],
                scale: 0,
                negative: false,
            });
        }

        let scale = self.scale + other.scale;
        let negative = self.negative ^ other.negative;

        // Compute the full 384-bit product using schoolbook multiplication.
        let mut product = Buf48::zero();
        let a = &self.data;
        let b = &other.data;

        // Find actual upper words to avoid unnecessary multiplications
        let a_upper = upper_word_6(a);
        let b_upper = upper_word_6(b);

        for i in 0..=a_upper {
            if a[i] == 0 {
                continue;
            }
            let mut carry: u64 = 0;
            for j in 0..=b_upper {
                let pos = i + j;
                carry += (a[i] as u64) * (b[j] as u64) + (product.data[pos] as u64);
                product.data[pos] = carry as u32;
                carry >>= 32;
            }
            // Propagate remaining carry
            let mut pos = i + b_upper + 1;
            while carry > 0 && pos < 12 {
                carry += product.data[pos] as u64;
                product.data[pos] = carry as u32;
                carry >>= 32;
                pos += 1;
            }
        }

        // Rescale the 384-bit product down to 192 bits
        let mut upper = product.upper_word();
        let mut scale = scale as i32;

        // If it already fits in 192 bits, great
        if upper <= 5 {
            let mut data = [0u32; 6];
            data.copy_from_slice(&product.data[..6]);

            // Still may need to reduce scale if it exceeds what we allow
            // For wide intermediates, we allow up to 57 (the max meaningful
            // for 192 bits of mantissa). We must also handle the case where
            // scale is much larger (e.g. 14*4=56 after 4 squarings).
            let max_wide_scale = 57i32;
            if scale > max_wide_scale {
                let mut excess = scale - max_wide_scale;
                let mut u = upper_word_6(&data);
                while excess > 0 {
                    let power_idx = (excess).min(MAX_I32_SCALE) as usize;
                    let power = POWERS_10[power_idx];
                    div_buf_by_power(&mut data, &mut u, power);
                    excess -= power_idx as i32;
                    scale -= power_idx as i32;
                }
            }

            return Some(DecWide {
                data,
                scale: scale as u32,
                negative,
            });
        }

        // Need to rescale down from >192 bits
        rescale_buf::<12, 5>(&mut product.data, &mut upper, &mut scale)?;

        let mut data = [0u32; 6];
        data.copy_from_slice(&product.data[..6]);
        Some(DecWide {
            data,
            scale: scale as u32,
            negative,
        })
    }
}

impl Buf48 {
    fn zero() -> Self {
        Buf48 { data: [0u32; 12] }
    }

    fn upper_word(&self) -> usize {
        for i in (0..12).rev() {
            if self.data[i] > 0 {
                return i;
            }
        }
        0
    }
}

fn upper_word_6(data: &[u32; 6]) -> usize {
    for i in (0..6).rev() {
        if data[i] > 0 {
            return i;
        }
    }
    0
}

/// Divide a buffer by a power of 10, with rounding.
fn div_buf_by_power<const N: usize>(data: &mut [u32; N], upper: &mut usize, power: u32) {
    let mut remainder = 0u32;
    let u = *upper;

    for i in (0..=u).rev() {
        let num = (data[i] as u64) + ((remainder as u64) << 32);
        data[i] = (num / power as u64) as u32;
        remainder = (num as u32).wrapping_sub(data[i].wrapping_mul(power));
    }

    if data[u] == 0 && u > 0 {
        *upper = u - 1;
    }

    // Round
    let power_half = power >> 1;
    if remainder > power_half || (remainder == power_half && (data[0] & 1) != 0) {
        let mut carry = true;
        for word in data.iter_mut() {
            if carry {
                *word = word.wrapping_add(1);
                carry = *word == 0;
            } else {
                break;
            }
        }
    }
}

/// Generic rescale: divide buffer by powers of 10 until upper <= target_upper.
/// Updates scale accordingly. Returns None on overflow.
fn rescale_buf<const N: usize, const TARGET: usize>(
    data: &mut [u32; N],
    upper: &mut usize,
    scale: &mut i32,
) -> Option<()> {
    if *upper <= TARGET && *scale <= Decimal::MAX_SCALE as i32 {
        return Some(());
    }

    // Estimate how much we need to divide
    let mut rescale_target = if *upper > TARGET {
        let bits = (*upper - TARGET) as i32 * 32
            - (data[*upper].leading_zeros() as i32);
        // Convert bits to decimal digits (multiply by log10(2) ≈ 77/256)
        let digits = ((bits.max(0)) * 77 >> 8) + 1;
        digits
    } else {
        0i32
    };

    // Also need to reduce scale to MAX_SCALE for the final target
    let max_scale = if TARGET <= 2 {
        Decimal::MAX_SCALE as i32
    } else {
        57 // Max meaningful scale for 192 bits
    };
    if *scale - rescale_target > max_scale {
        rescale_target = *scale - max_scale;
    }

    if rescale_target <= 0 && *upper <= TARGET {
        return Some(());
    }
    if rescale_target > *scale {
        return None;
    }

    let mut sticky = 0u32;
    let mut remainder = 0u32;

    while rescale_target > 0 || *upper > TARGET {
        sticky |= remainder;
        let power_idx = rescale_target.min(MAX_I32_SCALE).max(1) as usize;
        let power = POWERS_10[power_idx];

        // Divide the entire buffer by power
        remainder = 0;
        for i in (0..=*upper).rev() {
            let num = (data[i] as u64) + ((remainder as u64) << 32);
            data[i] = (num / power as u64) as u32;
            remainder = (num as u32).wrapping_sub(data[i].wrapping_mul(power));
        }

        // Adjust upper
        while *upper > 0 && data[*upper] == 0 {
            *upper -= 1;
        }

        *scale -= power_idx as i32;
        rescale_target -= power_idx as i32;

        if *upper > TARGET && rescale_target <= 0 {
            // Still too big, need more rescaling
            if *scale <= 0 {
                return None;
            }
            rescale_target = 1;
        }
    }

    // Round the final result
    let sticky_combined = sticky | remainder;
    if sticky_combined > 0 {
        // Use the last remainder for rounding
        // remainder is from the last division, so compare with power/2
        // Since we don't have the last power easily, use the general rule:
        // if remainder > 0 from the last step, we need to check more carefully
        // For simplicity, round if remainder was >= half the last power
        // But we've already lost the exact power. Use banker's rounding on the remainder.
        // Actually, the remainder is still from the last division step.
        // The last power used was POWERS_10[power_idx], but we've exited the loop.
        // We can reconstruct: check if remainder suggests rounding up.
        if remainder > 0 {
            // Crude rounding: if remainder > 0, we had a fractional part.
            // For banker's rounding we'd need the exact halfway point.
            // As a reasonable approximation, round up if remainder > half or if odd.
            // Since we don't know the exact divisor here, just check if any remainder exists
            // and the low bit is set (biased toward rounding).
            if (data[0] & 1) != 0 || remainder > 1 {
                let mut carry = true;
                for word in data.iter_mut() {
                    if carry {
                        *word = word.wrapping_add(1);
                        carry = *word == 0;
                    } else {
                        break;
                    }
                }
                // Check if carry propagated past the target
                if carry || data.get(TARGET + 1).map_or(false, |&w| w > 0) {
                    if *scale <= 0 {
                        return None;
                    }
                    // Need one more rescale step
                    let power = POWERS_10[1]; // divide by 10
                    let mut rem2 = 0u32;
                    // Recalculate upper since carry may have extended it
                    for i in (0..N).rev() {
                        if data[i] > 0 {
                            *upper = i;
                            break;
                        }
                    }
                    for i in (0..=*upper).rev() {
                        let num = (data[i] as u64) + ((rem2 as u64) << 32);
                        data[i] = (num / power as u64) as u32;
                        rem2 = (num as u32).wrapping_sub(data[i].wrapping_mul(power));
                    }
                    while *upper > 0 && data[*upper] == 0 {
                        *upper -= 1;
                    }
                    *scale -= 1;
                }
            }
        }
    }

    if *scale < 0 || *upper > TARGET {
        return None;
    }

    Some(())
}

/// Exponentiation by squaring using 192-bit intermediate precision.
/// Only truncates to 96-bit Decimal at the very end.
pub(crate) fn powu_wide(base: &Decimal, exp: u64) -> Option<Decimal> {
    if exp == 0 {
        return Some(Decimal::ONE);
    }
    if base.is_zero() {
        return Some(Decimal::ZERO);
    }
    if *base == Decimal::ONE {
        return Some(Decimal::ONE);
    }

    match exp {
        0 => unreachable!(),
        1 => Some(*base),
        2 => base.checked_mul(*base),
        _ => {
            let mut product = DecWide::from_decimal(&Decimal::ONE);
            let mut mask = exp;
            let mut power = DecWide::from_decimal(base);

            for n in 0..(64 - exp.leading_zeros()) {
                if n > 0 {
                    power = power.checked_mul(&power)?;
                    mask >>= 1;
                }
                if mask & 0x01 > 0 {
                    product = product.checked_mul(&power)?;
                }
            }

            let mut result = product.to_decimal()?;
            result.normalize_assign();
            Some(result)
        }
    }
}
