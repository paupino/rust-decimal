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

use crate::constants::POWERS_10;
use crate::Decimal;

/// Maximum power of 10 that fits in a u32 (10^9 = 1,000,000,000).
/// Duplicated here so `wide.rs` compiles regardless of `legacy-ops`.
const MAX_I32_SCALE: i32 = 9;

/// Extended precision decimal with 192-bit mantissa.
/// Used as an intermediate representation to avoid precision loss in
/// iterative operations.
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
    #[inline]
    const fn is_zero(&self) -> bool {
        let mut i = 0;
        while i < 6 {
            if self.data[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    #[inline]
    pub const fn from_decimal(d: &Decimal) -> Self {
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

        if upper <= 2 && scale <= Decimal::MAX_SCALE as i32 {
            return Some(Decimal::from_parts(
                data[0],
                data[1],
                data[2],
                self.negative,
                scale as u32,
            ));
        }

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
        if self.is_zero() || other.is_zero() {
            return Some(DecWide {
                data: [0; 6],
                scale: 0,
                negative: false,
            });
        }

        let scale = self.scale + other.scale;
        let negative = self.negative ^ other.negative;

        let mut product = Buf48 { data: [0u32; 12] };
        let a = &self.data;
        let b = &other.data;

        let a_upper = upper_word_6(a);
        let b_upper = upper_word_6(b);

        for (i, &a_word) in a.iter().enumerate().take(a_upper + 1) {
            if a_word == 0 {
                continue;
            }
            let mut carry: u64 = 0;
            for (j, &b_word) in b.iter().enumerate().take(b_upper + 1) {
                let pos = i + j;
                carry += (a_word as u64) * (b_word as u64) + (product.data[pos] as u64);
                product.data[pos] = carry as u32;
                carry >>= 32;
            }
            let mut pos = i + b_upper + 1;
            while carry > 0 && pos < 12 {
                carry += product.data[pos] as u64;
                product.data[pos] = carry as u32;
                carry >>= 32;
                pos += 1;
            }
        }

        let mut upper = product.upper_word();
        let mut scale = scale as i32;

        if upper <= 5 {
            let mut data = [0u32; 6];
            data.copy_from_slice(&product.data[..6]);

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

        rescale_buf::<12, 5>(&mut product.data, &mut upper, &mut scale)?;

        let mut data = [0u32; 6];
        data.copy_from_slice(&product.data[..6]);
        Some(DecWide {
            data,
            scale: scale as u32,
            negative,
        })
    }

    /// Add two DecWide values, keeping 192-bit precision.
    pub fn checked_add(&self, other: &DecWide) -> Option<DecWide> {
        if self.is_zero() {
            return Some(other.clone());
        }
        if other.is_zero() {
            return Some(self.clone());
        }

        if self.negative != other.negative {
            // a + (-b) = a - b: flip other's sign and subtract
            return self.checked_sub_impl(other, !other.negative);
        }

        // Same sign: align scales, then add mantissas
        let (mut a, mut b) = (self.clone(), other.clone());
        align_scales(&mut a, &mut b)?;

        let mut carry = 0u64;
        let mut data = [0u32; 6];
        for (dest, (&a_word, &b_word)) in data.iter_mut().zip(a.data.iter().zip(b.data.iter())) {
            carry += a_word as u64 + b_word as u64;
            *dest = carry as u32;
            carry >>= 32;
        }

        if carry > 0 {
            // Overflow 192 bits - divide by 10 to make room
            let mut buf = [0u32; 7];
            buf[..6].copy_from_slice(&data);
            buf[6] = carry as u32;
            let mut scale = a.scale as i32;
            let mut remainder = 0u32;
            for i in (0..7).rev() {
                let num = (buf[i] as u64) + ((remainder as u64) << 32);
                buf[i] = (num / 10) as u32;
                remainder = (num % 10) as u32;
            }
            scale -= 1;
            if scale < 0 {
                return None;
            }
            data.copy_from_slice(&buf[..6]);
            if remainder >= 5 {
                add_one(&mut data);
            }
            return Some(DecWide {
                data,
                scale: scale as u32,
                negative: a.negative,
            });
        }

        Some(DecWide {
            data,
            scale: a.scale,
            negative: a.negative,
        })
    }

    /// Core subtraction with explicit sign for `other`.
    fn checked_sub_impl(&self, other: &DecWide, other_negative: bool) -> Option<DecWide> {
        if other.is_zero() {
            return Some(self.clone());
        }
        if self.is_zero() {
            return Some(DecWide {
                data: other.data,
                scale: other.scale,
                negative: !other_negative,
            });
        }

        if self.negative != other_negative {
            // Different effective signs: a - (-b) = a + b
            let mut b = other.clone();
            b.negative = self.negative; // same sign as self
            return self.checked_add(&b);
        }

        // Same effective sign: align and subtract
        let (mut a, mut b_val) = (self.clone(), other.clone());
        b_val.negative = other_negative;
        align_scales(&mut a, &mut b_val)?;

        let a_bigger = cmp_data(&a.data, &b_val.data) != core::cmp::Ordering::Less;
        let (big, small, neg) = if a_bigger {
            (&a.data, &b_val.data, a.negative)
        } else {
            (&b_val.data, &a.data, !a.negative)
        };

        let mut borrow = 0i64;
        let mut data = [0u32; 6];
        for i in 0..6 {
            let diff = big[i] as i64 - small[i] as i64 - borrow;
            if diff < 0 {
                data[i] = (diff + (1i64 << 32)) as u32;
                borrow = 1;
            } else {
                data[i] = diff as u32;
                borrow = 0;
            }
        }

        Some(DecWide {
            data,
            scale: a.scale,
            negative: neg,
        })
    }

    /// Divide by a small u32 value (for Taylor series: divide by i).
    pub fn checked_div_u32(&self, divisor: u32) -> Option<DecWide> {
        if divisor == 0 {
            return None;
        }
        if self.is_zero() || divisor == 1 {
            return Some(self.clone());
        }

        let mut data = self.data;
        let mut remainder = 0u64;

        for i in (0..6).rev() {
            let num = (data[i] as u64) + (remainder << 32);
            data[i] = (num / divisor as u64) as u32;
            remainder = num % divisor as u64;
        }

        let mut scale = self.scale;

        if remainder > 0 {
            let upper = upper_word_6(&data);
            let used_bits = if upper == 0 && data[0] == 0 {
                0
            } else {
                upper * 32 + (32 - data[upper].leading_zeros() as usize)
            };
            let free_digits = (((192 - used_bits as i32) * 77) >> 8).max(0) as u32;
            let extra_scale = free_digits.min(9);

            if extra_scale > 0 && scale + extra_scale <= 57 {
                let power = POWERS_10[extra_scale as usize];
                let mut carry = 0u64;
                for word in data.iter_mut() {
                    carry += *word as u64 * power as u64;
                    *word = carry as u32;
                    carry >>= 32;
                }
                let rem_scaled = remainder * power as u64;
                let extra_quotient = rem_scaled / divisor as u64;
                remainder = rem_scaled % divisor as u64;
                let mut add_carry = extra_quotient;
                for word in data.iter_mut() {
                    add_carry += *word as u64;
                    *word = add_carry as u32;
                    add_carry >>= 32;
                    if add_carry == 0 {
                        break;
                    }
                }
                scale += extra_scale;
            }
        }

        // Round
        if remainder > 0 {
            let half = divisor as u64 / 2;
            if remainder > half || (remainder == half && (data[0] & 1) != 0) {
                add_one(&mut data);
            }
        }

        Some(DecWide {
            data,
            scale,
            negative: self.negative,
        })
    }

    /// Check if this value's magnitude is less than or equal to 1e-28.
    /// Uses a fast path that avoids the expensive to_decimal() rescale in most cases.
    #[inline]
    pub const fn magnitude_le_28(&self) -> bool {
        if self.is_zero() {
            return true;
        }
        // value = mantissa * 10^(-scale)
        // We want: mantissa * 10^(-scale) <= 10^(-28)
        // i.e.: mantissa <= 10^(scale - 28)
        //
        // Fast check: if the mantissa fits in one u32 word (< 4.3e9 < 10^10)
        // and scale >= 38, then value < 10^10 * 10^(-38) = 10^(-28). Done.
        //
        // If mantissa fits in two words (< 1.8e19 < 10^20)
        // and scale >= 48, then value < 10^20 * 10^(-48) = 10^(-28). Done.
        let upper = upper_word_6(&self.data);
        let min_scale = match upper {
            0 => 38,
            1 => 48,
            2 => 57,           // 10^29 < 2^97, so 3 words with scale >= 57 → value < 10^(-28)
            _ => return false, // Large mantissa, definitely > 1e-28
        };
        self.scale >= min_scale
    }

    /// Negate in place
    #[inline]
    pub fn negate(&mut self) {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
    }

    pub const fn one() -> DecWide {
        DecWide::from_decimal(&Decimal::ONE)
    }
}

#[inline]
fn add_one<const N: usize>(data: &mut [u32; N]) {
    let mut carry = 1u64;
    for word in data.iter_mut() {
        carry += *word as u64;
        *word = carry as u32;
        carry >>= 32;
        if carry == 0 {
            break;
        }
    }
}

fn align_scales(a: &mut DecWide, b: &mut DecWide) -> Option<()> {
    if a.scale == b.scale {
        return Some(());
    }

    let (smaller, larger_scale) = if a.scale < b.scale {
        (&mut *a, b.scale)
    } else {
        (&mut *b, a.scale)
    };

    let diff = larger_scale - smaller.scale;
    let mut remaining = diff;
    while remaining > 0 {
        let step = remaining.min(MAX_I32_SCALE as u32);
        let power = POWERS_10[step as usize];

        let mut carry = 0u64;
        for i in 0..6 {
            carry += smaller.data[i] as u64 * power as u64;
            smaller.data[i] = carry as u32;
            carry >>= 32;
        }

        if carry > 0 {
            return None;
        }

        smaller.scale += step;
        remaining -= step;
    }

    Some(())
}

const fn cmp_data(a: &[u32; 6], b: &[u32; 6]) -> core::cmp::Ordering {
    let mut i = 5;
    loop {
        if a[i] > b[i] {
            return core::cmp::Ordering::Greater;
        }
        if a[i] < b[i] {
            return core::cmp::Ordering::Less;
        }
        if i == 0 {
            return core::cmp::Ordering::Equal;
        }
        i -= 1;
    }
}

impl Buf48 {
    const fn upper_word(&self) -> usize {
        let mut i = 11;
        while i > 0 {
            if self.data[i] > 0 {
                return i;
            }
            i -= 1;
        }
        0
    }
}

const fn upper_word_6(data: &[u32; 6]) -> usize {
    let mut i = 5;
    while i > 0 {
        if data[i] > 0 {
            return i;
        }
        i -= 1;
    }
    0
}

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

    let power_half = power >> 1;
    if remainder > power_half || (remainder == power_half && (data[0] & 1) != 0) {
        add_one(data);
    }
}

fn rescale_buf<const N: usize, const TARGET: usize>(
    data: &mut [u32; N],
    upper: &mut usize,
    scale: &mut i32,
) -> Option<()> {
    if *upper <= TARGET && *scale <= Decimal::MAX_SCALE as i32 {
        return Some(());
    }

    let mut rescale_target = if *upper > TARGET {
        let bits = (*upper - TARGET) as i32 * 32 - (data[*upper].leading_zeros() as i32);
        (((bits.max(0)) * 77) >> 8) + 1
    } else {
        0i32
    };

    let max_scale = if TARGET <= 2 { Decimal::MAX_SCALE as i32 } else { 57 };
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
        let power_idx = rescale_target.clamp(1, MAX_I32_SCALE) as usize;
        let power = POWERS_10[power_idx];

        remainder = 0;
        for i in (0..=*upper).rev() {
            let num = (data[i] as u64) + ((remainder as u64) << 32);
            data[i] = (num / power as u64) as u32;
            remainder = (num as u32).wrapping_sub(data[i].wrapping_mul(power));
        }

        while *upper > 0 && data[*upper] == 0 {
            *upper -= 1;
        }

        *scale -= power_idx as i32;
        rescale_target -= power_idx as i32;

        if *upper > TARGET && rescale_target <= 0 {
            if *scale <= 0 {
                return None;
            }
            rescale_target = 1;
        }
    }

    let sticky_combined = sticky | remainder;
    if sticky_combined > 0 && remainder > 0 && ((data[0] & 1) != 0 || remainder > 1) {
        let mut carry = true;
        for word in data.iter_mut() {
            if carry {
                *word = word.wrapping_add(1);
                carry = *word == 0;
            } else {
                break;
            }
        }
        if carry || data.get(TARGET + 1).map_or(false, |&w| w > 0) {
            if *scale <= 0 {
                return None;
            }
            let power = POWERS_10[1];
            let mut rem2 = 0u32;
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

    if *scale < 0 || *upper > TARGET {
        return None;
    }

    Some(())
}

/// Exponentiation by squaring using adaptive precision.
///
/// For small exponents (fewer than 10 squarings, i.e. exp < 1024), uses
/// standard 96-bit Decimal arithmetic - fast and sufficient precision (~18+
/// correct digits). For large exponents, uses 192-bit DecWide intermediates
/// to prevent precision loss from compounding over many squarings.
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
        1 => Some(*base),
        2 => base.checked_mul(*base),
        _ => {
            // Number of squarings = bit_length - 1.
            // Each squaring in 96-bit loses ~1 decimal digit.
            // With ≤10 squarings (exp < 1024), we keep 18+ correct digits.
            let squarings = 63 - exp.leading_zeros();
            if squarings < 10 {
                powu_narrow(base, exp)
            } else {
                powu_192(base, exp)
            }
        }
    }
}

/// Fast path: exponentiation by squaring using 96-bit Decimal.
fn powu_narrow(base: &Decimal, exp: u64) -> Option<Decimal> {
    let mut product = Decimal::ONE;
    let mut mask = exp;
    let mut power = *base;

    for n in 0..(64 - exp.leading_zeros()) {
        if n > 0 {
            power = power.checked_mul(power)?;
            mask >>= 1;
        }
        if mask & 0x01 > 0 {
            product = product.checked_mul(power)?;
        }
    }

    product.normalize_assign();
    Some(product)
}

/// Precise path: exponentiation by squaring using 192-bit DecWide.
fn powu_192(base: &Decimal, exp: u64) -> Option<Decimal> {
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

/// Compute exp(x) using 192-bit intermediate precision.
///
/// Uses argument reduction: exp(x) = exp(n) * exp(r) where n = floor(x), r = x - n.
/// - exp(n) = e^n via powu squaring in DecWide
/// - exp(r) via Taylor series entirely in DecWide
pub(crate) fn exp_wide(value: &Decimal) -> Option<Decimal> {
    if value.is_zero() {
        return Some(Decimal::ONE);
    }
    if value.is_sign_negative() {
        let mut pos = *value;
        pos.set_sign_positive(true);
        let exp = exp_wide(&pos)?;
        return Decimal::ONE.checked_div(exp);
    }

    let n = value.floor();
    let r = value.checked_sub(n)?;

    // Compute exp(r) via Taylor series in DecWide precision
    let r_wide = DecWide::from_decimal(&r);
    let exp_r = if r.is_zero() {
        DecWide::from_decimal(&Decimal::ONE)
    } else {
        let one_wide = DecWide::from_decimal(&Decimal::ONE);
        let mut result = one_wide.checked_add(&r_wide)?;
        let mut term = r_wide.clone();

        for i in 2..100u32 {
            term = r_wide.checked_mul(&term.checked_div_u32(i)?)?;
            result = result.checked_add(&term)?;

            if term.magnitude_le_28() {
                break;
            }
        }
        result
    };

    if n.is_zero() {
        return exp_r.to_decimal();
    }

    let m = n.mantissa_array3();
    if m[2] != 0 {
        return None;
    }
    let n_u64 = m[0] as u64 + ((m[1] as u64) << 32);

    // Compute e^n in DecWide via squaring.
    // We use the 28-digit Decimal::E (not the 57-digit WIDE_E) because when
    // squared, 28 digits → 56 digits which fits perfectly in DecWide's 192
    // bits (~57.8 digits) with minimal truncation. Starting with 57 digits
    // would overflow to 114 digits on first squaring, losing half immediately.
    let exp_n = {
        let mut product = DecWide::from_decimal(&Decimal::ONE);
        let mut mask = n_u64;
        let mut power = DecWide::from_decimal(&Decimal::E);

        for i in 0..(64 - n_u64.leading_zeros()) {
            if i > 0 {
                power = power.checked_mul(&power)?;
                mask >>= 1;
            }
            if mask & 0x01 > 0 {
                product = product.checked_mul(&power)?;
            }
        }
        product
    };

    let result_wide = exp_n.checked_mul(&exp_r)?;
    let mut result = result_wide.to_decimal()?;
    result.normalize_assign();
    Some(result)
}

/// Compute ln(x) using 192-bit intermediate precision.
///
/// Uses range reduction (multiply/divide by e), then the atanh series:
/// ln(x) = 2 * atanh((x-1)/(x+1)) where atanh(z) = z + z³/3 + z⁵/5 + ...
/// This converges much faster than the standard ln(1+t) series.
pub(crate) fn ln_wide(value: &Decimal) -> Option<Decimal> {
    if value.is_sign_negative() || value.is_zero() {
        return None;
    }
    if *value == Decimal::ONE {
        return Some(Decimal::ZERO);
    }

    // Range reduction: multiply/divide by e until value is in (e^-1, 1]
    let mut x = *value;
    let mut count: i32 = 0;
    while x >= Decimal::ONE {
        x *= Decimal::E_INVERSE;
        count += 1;
    }
    while x <= Decimal::E_INVERSE {
        x *= Decimal::E;
        count -= 1;
    }

    // x is in (e^-1, 1], compute z = (x-1)/(x+1) in wide precision
    let x_wide = DecWide::from_decimal(&x);
    let one_wide = DecWide::one();
    let x_minus_1 = x_wide.checked_sub_impl(&one_wide, false)?;
    if x_minus_1.is_zero() {
        return Some(Decimal::new(count as i64, 0));
    }
    let x_plus_1 = x_wide.checked_add(&one_wide)?;

    // z = (x-1)/(x+1): compute via wide division (multiply by reciprocal approximation)
    // Since we don't have wide division, convert to Decimal for this one division
    let x_m1_dec = x_minus_1.to_decimal()?;
    let x_p1_dec = x_plus_1.to_decimal()?;
    let z_dec = x_m1_dec.checked_div(x_p1_dec)?;

    let z = DecWide::from_decimal(&z_dec);
    let z2 = z.checked_mul(&z)?;

    // atanh(z) = z + z³/3 + z⁵/5 + z⁷/7 + ...
    let mut result = z.clone();
    let mut term = z;

    for n in 1..100u32 {
        let denom = 2 * n + 1;
        term = term.checked_mul(&z2)?;
        let contribution = term.checked_div_u32(denom)?;
        result = result.checked_add(&contribution)?;

        if contribution.magnitude_le_28() {
            break;
        }
    }

    // ln(x) = 2 * atanh(z)
    let two = DecWide::from_decimal(&Decimal::TWO);
    let ln_x = two.checked_mul(&result)?;

    // ln(value) = count + ln(x)
    let ln_fractional = ln_x.to_decimal()?;
    let mut out = Decimal::new(count as i64, 0).checked_add(ln_fractional)?;
    out.normalize_assign();
    Some(out)
}

/// Compute sin(x) using 192-bit intermediate precision.
pub(crate) fn sin_wide(value: &Decimal) -> Option<Decimal> {
    if value.is_zero() {
        return Some(Decimal::ZERO);
    }
    if value.is_sign_negative() {
        return sin_wide(&(-*value)).map(|x| -x);
    }
    if *value >= Decimal::TWO_PI {
        let adjusted = value.checked_rem(Decimal::TWO_PI)?;
        return sin_wide(&adjusted);
    }
    if *value >= Decimal::PI {
        return sin_wide(&(*value - Decimal::PI)).map(|x| -x);
    }
    if *value > Decimal::QUARTER_PI {
        return cos_wide(&(Decimal::HALF_PI - *value));
    }

    let x_wide = DecWide::from_decimal(value);
    let x2 = x_wide.checked_mul(&x_wide)?;

    // sin(x) = x - x³/3! + x⁵/5! - ...
    // term_{n+1} = -term_n * x² / ((2n+2)(2n+3))
    let mut result = x_wide.clone();
    let mut term = x_wide;

    for n in 0..50u32 {
        let d = (2 * n + 2) * (2 * n + 3);
        term = term.checked_mul(&x2)?.checked_div_u32(d)?;
        term.negate();
        result = result.checked_add(&term)?;

        if term.magnitude_le_28() {
            break;
        }
    }

    let mut out = result.to_decimal()?;
    out.normalize_assign();
    Some(out)
}

/// Compute cos(x) using 192-bit intermediate precision.
pub(crate) fn cos_wide(value: &Decimal) -> Option<Decimal> {
    if value.is_zero() {
        return Some(Decimal::ONE);
    }
    if value.is_sign_negative() {
        return cos_wide(&(-*value));
    }
    if *value >= Decimal::TWO_PI {
        let adjusted = value.checked_rem(Decimal::TWO_PI)?;
        return cos_wide(&adjusted);
    }
    if *value >= Decimal::PI {
        return cos_wide(&(*value - Decimal::PI)).map(|x| -x);
    }
    if *value > Decimal::QUARTER_PI {
        return sin_wide(&(Decimal::HALF_PI - *value));
    }

    let x_wide = DecWide::from_decimal(value);
    let x2 = x_wide.checked_mul(&x_wide)?;

    // cos(x) = 1 - x²/2! + x⁴/4! - ...
    // term_{n+1} = -term_n * x² / ((2n+1)(2n+2))
    let mut result = DecWide::one();
    let mut term = DecWide::one();

    for n in 0..50u32 {
        let d = (2 * n + 1) * (2 * n + 2);
        term = term.checked_mul(&x2)?.checked_div_u32(d)?;
        term.negate();
        result = result.checked_add(&term)?;

        if term.magnitude_le_28() {
            break;
        }
    }

    let mut out = result.to_decimal()?;
    out.normalize_assign();
    Some(out)
}
