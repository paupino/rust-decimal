use Error;
use num::{BigInt, BigUint, FromPrimitive, One, ToPrimitive, Zero};
use num::bigint::Sign::{Minus, Plus};
use num::bigint::ToBigInt;
use std::cmp::*;
use std::cmp::Ordering::Equal;
use std::fmt;
use std::iter::repeat;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use std::str::FromStr;

// Sign mask for the flags field. A value of zero in this bit indicates a
// positive Decimal value, and a value of one in this bit indicates a
// negative Decimal value.
const SIGN_MASK: u32 = 0x8000_0000;

// Scale mask for the flags field. This byte in the flags field contains
// the power of 10 to divide the Decimal value by. The scale byte must
// contain a value between 0 and 28 inclusive.
const SCALE_MASK: u32 = 0x00FF_0000;
const U8_MASK: u32 = 0x0000_00FF;
const I32_MASK: u64 = 0xFFFF_FFFF;

// Number of bits scale is shifted by.
const SCALE_SHIFT: u32 = 16;

// The maximum supported precision
const MAX_PRECISION: u32 = 28;
const MAX_BYTES: usize = 12;

static ONE_INTERNAL_REPR: [u32; 3] = [1, 0, 0];

lazy_static! {
    static ref MIN: Decimal = Decimal {
        flags: 2_147_483_648,
        lo: 4_294_967_295,
        mid: 4_294_967_295,
        hi: 4_294_967_295
    };
    static ref MAX: Decimal = Decimal {
        flags: 0,
        lo: 4_294_967_295,
        mid: 4_294_967_295,
        hi: 4_294_967_295
    };
}

// Fast access for 10^n where n is 0-9
static POWERS_10: [u32; 10] = [
    1,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
];
// Fast access for 10^n where n is 10-19
static BIG_POWERS_10: [u64; 10] = [
    10000000000,
    100000000000,
    1000000000000,
    10000000000000,
    100000000000000,
    1000000000000000,
    10000000000000000,
    100000000000000000,
    1000000000000000000,
    10000000000000000000,
];

/// `Decimal` represents a 128 bit representation of a fixed-precision decimal number.
/// The finite set of values of type `Decimal` are of the form m / 10^e,
/// where m is an integer such that -2^96 <= m <= 2^96, and e is an integer
/// between 0 and 28 inclusive.
#[derive(Clone, Debug, Copy)]
pub struct Decimal {
    // Bits 0-15: unused
    // Bits 16-23: Contains "e", a value between 0-28 that indicates the scale
    // Bits 24-30: unused
    // Bit 31: the sign of the Decimal value, 0 meaning positive and 1 meaning negative.
    flags: u32,
    // The lo, mid, hi, and flags fields contain the representation of the
    // Decimal value as a 96-bit integer.
    hi: u32,
    lo: u32,
    mid: u32,
}

#[allow(dead_code)]
impl Decimal {
    /// Returns a `Decimal` with a 64 bit `m` representation and corresponding `e` scale.
    ///
    /// # Arguments
    ///
    /// * `num` - An i64 that represents the `m` portion of the decimal number
    /// * `scale` - A u32 representing the `e` portion of the decimal number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    /// let pi = Decimal::new(3141i64, 3u32);
    /// ```
    pub fn new(num: i64, scale: u32) -> Decimal {
        if scale > MAX_PRECISION {
            panic!(
                "Scale exceeds the maximum precision allowed: {} > {}",
                scale,
                MAX_PRECISION
            );
        }
        let flags: u32 = scale << SCALE_SHIFT;
        if num < 0 {
            return Decimal {
                flags: flags | SIGN_MASK,
                hi: 0,
                lo: (num.abs() as u64 & I32_MASK) as u32,
                mid: ((num.abs() as u64 >> 32) & I32_MASK) as u32,
            };
        }
        Decimal {
            flags: flags,
            hi: 0,
            lo: (num as u64 & I32_MASK) as u32,
            mid: ((num as u64 >> 32) & I32_MASK) as u32,
        }
    }

    /// Returns the scale of the decimal number, otherwise known as `e`.
    pub fn scale(&self) -> u32 {
        ((self.flags & SCALE_MASK) >> SCALE_SHIFT) as u32
    }

    /// An optimized method for changing the sign of a decimal number.
    ///
    /// # Arguments
    ///
    /// * `positive`: true if the resulting decimal should be positive.
    pub fn set_sign(&mut self, positive: bool) {
        if positive {
            if self.is_negative() {
                self.flags ^= SIGN_MASK;
            }
        } else {
            self.flags |= SIGN_MASK;
        }
    }

    /// Returns a serialized version of the decimal number.
    /// The resulting byte array will have the following representation:
    ///
    /// * Bytes 1-4: flags
    /// * Bytes 5-8: lo portion of `m`
    /// * Bytes 9-12: mid portion of `m`
    /// * Bytes 13-16: high portion of `m`
    pub fn serialize(&self) -> [u8; 16] {
        [
            (self.flags & U8_MASK) as u8,
            ((self.flags >> 8) & U8_MASK) as u8,
            ((self.flags >> 16) & U8_MASK) as u8,
            ((self.flags >> 24) & U8_MASK) as u8,
            (self.lo & U8_MASK) as u8,
            ((self.lo >> 8) & U8_MASK) as u8,
            ((self.lo >> 16) & U8_MASK) as u8,
            ((self.lo >> 24) & U8_MASK) as u8,
            (self.mid & U8_MASK) as u8,
            ((self.mid >> 8) & U8_MASK) as u8,
            ((self.mid >> 16) & U8_MASK) as u8,
            ((self.mid >> 24) & U8_MASK) as u8,
            (self.hi & U8_MASK) as u8,
            ((self.hi >> 8) & U8_MASK) as u8,
            ((self.hi >> 16) & U8_MASK) as u8,
            ((self.hi >> 24) & U8_MASK) as u8,
        ]
    }

    /// Deserializes the given bytes into a decimal number.
    /// The deserialized byte representation must be 16 bytes and adhere to the followign convention:
    ///
    /// * Bytes 1-4: flags
    /// * Bytes 5-8: lo portion of `m`
    /// * Bytes 9-12: mid portion of `m`
    /// * Bytes 13-16: high portion of `m`
    pub fn deserialize(bytes: [u8; 16]) -> Decimal {
        Decimal {
            flags: u32::from(bytes[0]) | u32::from(bytes[1]) << 8 | u32::from(bytes[2]) << 16 |
                u32::from(bytes[3]) << 24,
            lo: u32::from(bytes[4]) | u32::from(bytes[5]) << 8 | u32::from(bytes[6]) << 16 | u32::from(bytes[7]) << 24,
            mid: u32::from(bytes[8]) | u32::from(bytes[9]) << 8 | u32::from(bytes[10]) << 16 |
                u32::from(bytes[11]) << 24,
            hi: u32::from(bytes[12]) | u32::from(bytes[13]) << 8 | u32::from(bytes[14]) << 16 |
                u32::from(bytes[15]) << 24,
        }
    }

    /// Returns `true` if the decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.flags & SIGN_MASK > 0
    }

    /// Returns `true` if the decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.flags & SIGN_MASK == 0
    }

    /// Returns the minimum possible number that `Decimal` can represent.
    pub fn min_value() -> Decimal {
        *MIN
    }

    /// Returns the maximum possible number that `Decimal` can represent.
    pub fn max_value() -> Decimal {
        *MAX
    }

    /// Returns a new `Decimal` number with no fractional portion (i.e. an integer).
    /// Rounding currently follows "Bankers Rounding" rules. e.g. 6.5 -> 6, 7.5 -> 8
    pub fn round(&self) -> Decimal {
        self.round_dp(0)
    }

    /// Returns a new `Decimal` number with the specified number of decimal points for fractional portion.
    /// Rounding currently follows "Bankers Rounding" rules. e.g. 6.5 -> 6, 7.5 -> 8
    ///
    /// # Arguments
    /// * `dp`: the number of decimal points to round to.
    pub fn round_dp(&self, dp: u32) -> Decimal {

        let old_scale = self.scale();

        // We artificially cap at 20 because of fast power lookup.
        // We should change this as it's not necessary.
        if dp < old_scale && dp < 20 {
            // Short circuit for zero
            if self.is_zero() {
                return self.rescale(dp);
            }

            // Check to see if we need to add or subtract one.
            // Some expected results assuming dp = 2 and old_scale = 3:
            //   1.235  = 1.24
            //   1.2361 = 1.24
            //   1.2250 = 1.22
            //   1.2251 = 1.23
            // If we consider this example, we have the following number in `low`:
            //   1235 (scale 3)
            //   12361
            //   12250
            //   12251
            let index = dp as usize;
            let power10 = if dp < 10 {
                Decimal::from_u32(POWERS_10[index]).unwrap()
            } else {
                Decimal::from_u64(BIG_POWERS_10[index - 10]).unwrap()
            };
            let mut value = self.mul(power10);

            // Do some midpoint rounding checks
            // We're actually doing two things here.
            //  1. Figuring out midpoint rounding when we're right on the boundary. e.g. 2.50000
            //  2. Figuring out whether to add one or not e.g. 2.51
            // We only need to search back a certain number. e.g. 2.500, round(2) search 1.
            let raw = self.to_biguint();

            // Get the decimal portion
            //  e.g. 2.5001, round(2) decimal portion = 01
            let offset = self.rescale(dp).rescale(old_scale).to_biguint();
            let decimal_portion = raw - offset;

            // Rescale to zero so it's easier to work with
            value = value.rescale(0u32);

            // If the decimal_portion is zero then we round based on the other data
            let mut cap = BigUint::from_u32(5u32).unwrap();
            for _ in 0..(old_scale - dp - 1) {
                cap = cap.mul(BigUint::from_u32(10u32).unwrap());
            }
            if decimal_portion == cap {
                let even_or_odd = value.rem(Decimal::from_u32(2u32).unwrap());
                if !even_or_odd.is_zero() {
                    value = value.add(Decimal::one());
                }
            } else if decimal_portion > cap {
                // Doesn't matter about the decimal portion
                if self.is_negative() {
                    value = value.sub(Decimal::one());
                } else {
                    value = value.add(Decimal::one());
                }
            }

            // Divide by the power to get back
            value.div(power10)
        } else {
            *self
        }
    }

    pub(crate) fn rescale(&self, exp: u32) -> Decimal {
        if exp > MAX_PRECISION {
            panic!("Cannot have an exponent greater than {}", MAX_PRECISION);
        }
        let diff = exp as i32 - self.scale() as i32;
        if diff == 0 {
            // Since it's a copy type we can just return the self
            return *self;
        }

        // 1.23 is scale 2. If we're making it 1.2300 scale 4
        // Raw bit manipulation is hard (going up is easy, going down is hard)
        // Let's just use BigUint to help out
        let unsigned = self.to_biguint();
        let result: BigUint;

        // Figure out whether to multiply or divide
        let power = power_10(diff.abs() as usize);
        if diff > 0 {
            result = unsigned * power;
        } else {
            result = unsigned / power;
        }

        // Convert it back
        let bytes = result.to_bytes_le();
        Decimal::from_bytes_le(bytes, exp, self.is_negative())
    }

    fn base2_to_decimal(bits: &mut [u32; 3], exponent2: i32, positive: bool, is64: bool) -> Option<Self> {
        // 2^exponent2 = (10^exponent2)/(5^exponent2)
        //             = (5^-exponent2)*(10^exponent2)
        let mut exponent5 = -exponent2;
        let mut exponent10 = exponent2; // Ultimately, we want this for the scale

        while exponent5 > 0 {
            // Check to see if the mantissa is divisible by 2
            if bits[0] & 0x1 == 0 {
                exponent10 += 1;
                exponent5 -= 1;

                // We can divide by 2 without losing precision
                let hi_carry = bits[2] & 0x1 == 1;
                bits[2] >>= 1;
                let mid_carry = bits[1] & 0x1 == 1;
                bits[1] = (bits[1] >> 1) | if hi_carry { SIGN_MASK } else { 0 };
                bits[0] = (bits[0] >> 1) | if mid_carry { SIGN_MASK } else { 0 };
            } else {
                // The mantissa is NOT divisible by 2. Therefore the mantissa should
                // be multiplied by 5, unless the multiplication overflows.
                exponent5 -= 1;

                let mut temp = [bits[0], bits[1], bits[2]];
                if mul_by_u32(&mut temp, 5) == 0 {
                    // Multiplication succeeded without overflow, so copy result back
                    bits[0] = temp[0];
                    bits[1] = temp[1];
                    bits[2] = temp[2];
                } else {
                    // Multiplication by 5 overflows. The mantissa should be divided
                    // by 2, and therefore will lose significant digits.
                    exponent10 += 1;

                    // Shift right
                    let hi_carry = bits[2] & 0x1 == 1;
                    bits[2] >>= 1;
                    let mid_carry = bits[1] & 0x1 == 1;
                    bits[1] = (bits[1] >> 1) | if hi_carry { SIGN_MASK } else { 0 };
                    bits[0] = (bits[0] >> 1) | if mid_carry { SIGN_MASK } else { 0 };
                }
            }
        }

        // In order to divide the value by 5, it is best to multiply by 2/10.
        // Therefore, exponent10 is decremented, and the mantissa should be multiplied by 2
        while exponent5 < 0 {
            if bits[2] & SIGN_MASK == 0 {
                // No far left bit, the mantissa can withstand a shift-left without overflowing
                exponent10 -= 1;
                exponent5 += 1;
                shl_internal(bits, 1);
            } else {
                // The mantissa would overflow if shifted. Therefore it should be
                // directly divided by 5. This will lose significant digits, unless
                // by chance the mantissa happens to be divisible by 5.
                exponent5 += 1;
                div_by_u32(bits, 5);
            }
        }

        // At this point, the mantissa has assimilated the exponent5, but
        // exponent10 might not be suitable for assignment. exponent10 must be
        // in the range [-MAX_PRECISION..0], so the mantissa must be scaled up or
        // down appropriately.
        while exponent10 > 0 {
            // In order to bring exponent10 down to 0, the mantissa should be
            // multiplied by 10 to compensate. If the exponent10 is too big, this
            // will cause the mantissa to overflow.
            if mul_by_u32(bits, 10) == 0 {
                exponent10 -= 1;
            } else {
                // Overflowed - return?
                return None;
            }
        }

        // In order to bring exponent up to -MAX_PRECISION, the mantissa should
        // be divided by 10 to compensate. If the exponent10 is too small, this
        // will cause the mantissa to underflow and become 0.
        while exponent10 < -(MAX_PRECISION as i32) {
            let rem10 = div_by_u32(bits, 10);
            exponent10 += 1;
            if is_all_zero(bits) {
                // Underflow, unable to keep dividing
                exponent10 = 0;
            } else if rem10 >= 5 {
                add_internal(bits, &ONE_INTERNAL_REPR);
            }
        }

        // This step is required in order to remove excess bits of precision from the
        // end of the bit representation, down to the precision guaranteed by the
        // floating point number
        if is64 {
            // Guaranteed to about 16 dp
            while exponent10 < 0 && (bits[2] != 0 || (bits[1] & 0xFFE0_0000) != 0) {

                let rem10 = div_by_u32(bits, 10);
                exponent10 += 1;
                if rem10 >= 5 {
                    add_internal(bits, &ONE_INTERNAL_REPR);
                }
            }
        } else {
            // Guaranteed to about 7 dp
            while exponent10 < 0 &&
                (bits[2] != 0 || bits[1] != 0 || (bits[2] == 0 && bits[1] == 0 && (bits[0] & 0xFF00_0000) != 0))
            {

                let rem10 = div_by_u32(bits, 10);
                exponent10 += 1;
                if rem10 >= 5 {
                    add_internal(bits, &ONE_INTERNAL_REPR);
                }
            }
        }

        // Remove multiples of 10 from the representation
        while exponent10 < 0 {
            let mut temp = [bits[0], bits[1], bits[2]];
            let remainder = div_by_u32(&mut temp, 10);
            if remainder == 0 {
                exponent10 += 1;
                bits[0] = temp[0];
                bits[1] = temp[1];
                bits[2] = temp[2];
            } else {
                break;
            }
        }

        // Scale assignment
        let mut flags: u32 = (-exponent10 as u32) << SCALE_SHIFT;
        if !positive {
            flags |= SIGN_MASK;
        }
        Some(Decimal {
            lo: bits[0],
            mid: bits[1],
            hi: bits[2],
            flags: flags,
        })
    }

    //
    // These do not address scale. If you want that, rescale to 0 first.
    //
    pub(crate) fn to_biguint(&self) -> BigUint {
        let bytes = self.unsigned_bytes_le();
        BigUint::from_bytes_le(&bytes[..])
    }

    fn to_bigint(&self) -> BigInt {
        let bytes = self.unsigned_bytes_le();
        let sign = if self.is_negative() { Minus } else { Plus };
        BigInt::from_bytes_le(sign, &bytes[..])
    }

    pub(crate) fn from_biguint(res: BigUint, scale: u32, negative: bool) -> Result<Decimal, Error> {
        let bytes = res.to_bytes_le();
        if bytes.len() > MAX_BYTES {
            return Err(Error::new("Decimal Overflow"));
        }
        if scale > MAX_PRECISION {
            return Err(Error::new("Scale exceeds maximum precision"));
        }

        Ok(Decimal::from_bytes_le(bytes, scale, negative))
    }

    fn unsigned_bytes_le(&self) -> Vec<u8> {
        return vec![
            (self.lo & U8_MASK) as u8,
            ((self.lo >> 8) & U8_MASK) as u8,
            ((self.lo >> 16) & U8_MASK) as u8,
            ((self.lo >> 24) & U8_MASK) as u8,
            (self.mid & U8_MASK) as u8,
            ((self.mid >> 8) & U8_MASK) as u8,
            ((self.mid >> 16) & U8_MASK) as u8,
            ((self.mid >> 24) & U8_MASK) as u8,
            (self.hi & U8_MASK) as u8,
            ((self.hi >> 8) & U8_MASK) as u8,
            ((self.hi >> 16) & U8_MASK) as u8,
            ((self.hi >> 24) & U8_MASK) as u8,
        ];
    }

    fn from_bytes_le(bytes: Vec<u8>, scale: u32, negative: bool) -> Decimal {
        // Finally build the flags
        let mut flags = 0u32;
        let mut lo = 0u32;
        let mut mid = 0u32;
        let mut hi = 0u32;

        if scale > 0 {
            flags = scale << SCALE_SHIFT;
        }
        if negative {
            flags |= SIGN_MASK;
        }
        if bytes.len() > MAX_BYTES {
            panic!(
                "Decimal Overflow, too many bytes {} > MAX({})",
                bytes.len(),
                MAX_BYTES
            );
        }

        let mut pos = 0;
        for b in bytes {
            if pos < 4 {
                lo |= u32::from(b) << (pos * 8);
            } else if pos < 8 {
                mid |= u32::from(b) << ((pos - 4) * 8);
            } else {
                hi |= u32::from(b) << ((pos - 8) * 8);
            }
            // Move position
            pos += 1;
        }

        // Build up each hi/lo
        Decimal {
            flags: flags,
            hi: hi,
            lo: lo,
            mid: mid,
        }
    }
}

fn power_10(exponent: usize) -> BigUint {
    if exponent < 10 {
        BigUint::from_u32(POWERS_10[exponent]).unwrap()
    } else if exponent < 20 {
        BigUint::from_u64(BIG_POWERS_10[exponent - 10]).unwrap()
    } else {
        let u32_exponent = exponent - 19; // -20 + 1 for getting the right u32 index
        BigUint::from_u64(BIG_POWERS_10[9]).unwrap() * BigUint::from_u32(POWERS_10[u32_exponent]).unwrap()
    }
}

fn copy_array(into: &mut [u32], from: &[u32]) {
    copy_array_with_limit(into, from, 0);
}

fn copy_array_with_limit(into: &mut [u32], from: &[u32], limit: usize) {
    let limit = if limit == 0 {
        from.len()
    } else {
        from.len().min(limit)
    };
    for i in 0..into.len() {
        if i >= limit {
            break;
        }
        into[i] = from[i];
    }
}

fn add_internal(value: &mut [u32], by: &[u32]) -> u32 {
    let mut carry: u64 = 0;
    let vl = value.len();
    let bl = by.len();
    if vl >= bl {
        let mut sum: u64;
        for i in 0..bl {
            sum = u64::from(value[i]) + u64::from(by[i]) + carry;
            value[i] = (sum & 0xFFFF_FFFF) as u32;
            carry = sum >> 32;
        }
        if vl > bl {
            for i in bl..vl {
                if carry == 0 {
                    break;
                }
                sum = u64::from(value[i]) + carry;
                value[i] = (sum & 0xFFFF_FFFF) as u32;
                carry = sum >> 32;
            }
        }
    }
    carry as u32
}

fn add_with_scale_internal(
    quotient: &mut [u32],
    quotient_scale: &mut i32,
    working: &mut [u32],
    working_scale: &mut i32,
) -> bool {
    // Add quotient and the working (i.e. quotient = quotient + working)
    // We only care about the first 4 words of working_quotient as we are only dealing with the quotient
    if is_all_zero(quotient) {
        // Quotient is zero (i.e. quotient = 0 + working_quotient).
        // We can just copy the working quotient in directly
        // First, make sure they are both 96 bit.
        while working[3] != 0 {
            div_by_u32(working, 10);
            *working_scale -= 1;
        }
        copy_array(quotient, working);
        *quotient_scale = *working_scale;
    } else if !is_some_zero(working, 0, 4) {
        // We have ensured that working is not zero so we should do the addition
        let mut temp = [0u32, 0u32, 0u32, 0u32, 0u32];

        // If our two quotients are different then
        // try to scale down the one with the bigger scale
        if *quotient_scale != *working_scale {
            if *quotient_scale < *working_scale {
                // divide by 10 until target scale is reached
                copy_array_with_limit(&mut temp, working, 4);
                while *working_scale > *quotient_scale {
                    // TODO: Work out a better way to share this code
                    let remainder = div_by_u32(&mut temp, 10);
                    if remainder == 0 {
                        *working_scale -= 1;
                        copy_array_with_limit(working, &temp, 4);
                    } else {
                        break;
                    }
                }
            } else {
                copy_array(&mut temp, quotient);
                // divide by 10 until target scale is reached
                while *quotient_scale > *working_scale {
                    // TODO: Work out a better way to share this code
                    let remainder = div_by_u32(&mut temp, 10);
                    if remainder == 0 {
                        *quotient_scale -= 1;
                        copy_array(quotient, &temp);
                    } else {
                        break;
                    }
                }

            }
        }

        // If our two quotients are still different then
        // try to scale up the smaller scale
        if *quotient_scale != *working_scale {
            if *quotient_scale > *working_scale {
                copy_array_with_limit(&mut temp, working, 4);
                // Multiply by 10 until scale reached or overflow
                while *working_scale < *quotient_scale && temp[4] == 0 {
                    mul_by_u32(&mut temp, 10);
                    if temp[4] == 0 {
                        // still does not overflow
                        *working_scale += 1;
                        copy_array_with_limit(working, &temp, 4);
                    }
                }
            } else {
                copy_array(&mut temp, quotient);
                // Multiply by 10 until scale reached or overflow
                while *quotient_scale < *working_scale && temp[3] == 0 {
                    mul_by_u32(&mut temp, 10);
                    if temp[3] == 0 {
                        // still does not overflow
                        *quotient_scale += 1;
                        copy_array(quotient, &temp);
                    }
                }
            }
        }

        // If our two quotients are still different then
        // try to scale down the one with the bigger scale
        // (ultimately losing significant digits)
        if *quotient_scale != *working_scale {
            if *quotient_scale < *working_scale {
                copy_array_with_limit(&mut temp, working, 4);
                // divide by 10 until target scale is reached
                while *working_scale > *quotient_scale {
                    div_by_u32(&mut temp, 10);
                    *working_scale -= 1;
                    copy_array_with_limit(working, &temp, 4);
                }

            } else {
                copy_array(&mut temp, quotient);
                // divide by 10 until target scale is reached
                while *quotient_scale > *working_scale {
                    div_by_u32(&mut temp, 10);
                    *quotient_scale -= 1;
                    copy_array(quotient, &temp);
                }
            }
        }

        // If quotient or working are zero we have an underflow condition
        if is_all_zero(quotient) || is_some_zero(working, 0, 4) {
            // Underflow
            return true;
        } else {
            // Both numbers have the same scale and can be added.
            // We just need to know whether we can fit them in
            let mut underflow = false;
            while !underflow {
                for i in 0..5 {
                    if i < 3 {
                        temp[i] = quotient[i];
                    } else {
                        temp[i] = 0;
                    }
                }

                let mut carry = 0;
                let mut sum: u64;
                for i in 0..4 {
                    sum = u64::from(temp[i]) + u64::from(working[i]) + carry as u64;
                    temp[i] = (sum & 0xFFFF_FFFF) as u32;
                    carry = sum >> 32;
                }
                sum = u64::from(temp[4]) + carry as u64;
                temp[4] = (sum & 0xFFFF_FFFF) as u32;

                if temp[3] == 0 && temp[4] == 0 {
                    // addition was successful
                    copy_array(quotient, &temp);
                    break;
                } else {
                    // addition overflowed - remove significant digits and try again
                    div_by_u32(quotient, 10);
                    *quotient_scale -= 1;
                    div_by_u32(working, 10);
                    *working_scale -= 1;
                    // Check for underflow
                    underflow = is_all_zero(quotient) || is_some_zero(working, 0, 4);
                }
            }
            if underflow {
                return true;
            }
        }
    }
    false
}

fn sub_internal(value: &mut [u32], by: &[u32]) -> u32 {
    // The way this works is similar to long subtraction
    // Let's assume we're working with bytes for simpliciy in an example:
    //   257 - 8 = 249
    //   0000_0001 0000_0001 - 0000_0000 0000_1000 = 0000_0000 1111_1001
    // We start by doing the first byte...
    //   Overflow = 0
    //   Left = 0000_0001 (1)
    //   Right = 0000_1000 (8)
    // Firstly, we make sure the left and right are scaled up to twice the size
    //   Left = 0000_0000 0000_0001
    //   Right = 0000_0000 0000_1000
    // We then subtract right from left
    //   Result = Left - Right = 1111_1111 1111_1001
    // We subtract the overflow, which in this case is 0.
    // Because left < right (1 < 8) we invert the high part.
    //   Lo = 1111_1001
    //   Hi = 1111_1111 -> 0000_0001
    // Lo is the field, hi is the overflow.
    // We do the same for the second byte...
    //   Overflow = 1
    //   Left = 0000_0001
    //   Right = 0000_0000
    //   Result = Left - Right = 0000_0000 0000_0001
    // We subtract the overflow...
    //   Result = 0000_0000 0000_0001 - 1 = 0
    // And we invert the high, just because (invert 0 = 0).
    // So our result is:
    //   0000_0000 1111_1001
    let mut overflow = 0;
    let vl = value.len();
    let bl = by.len();
    for i in 0..vl {
        if i >= bl {
            break;
        }
        let (lo, hi) = sub_part(value[i], by[i], overflow);
        value[i] = lo;
        overflow = hi;
    }
    overflow
}

fn sub_part(left: u32, right: u32, overflow: u32) -> (u32, u32) {
    let mut invert = false;
    let overflow = i64::from(overflow);
    let mut part: i64 = i64::from(left) - i64::from(right);
    if left < right {
        invert = true;
    }

    if part > overflow {
        part -= overflow;
    } else {
        part -= overflow;
        invert = true;
    }

    let mut hi: i32 = ((part >> 32) & 0xFFFF_FFFF) as i32;
    let lo: u32 = (part & 0xFFFF_FFFF) as u32;
    if invert {
        hi = -hi;
    }
    (lo, hi as u32)
}

// Returns overflow
fn mul_by_u32(bits: &mut [u32], m: u32) -> u32 {
    let mut overflow = 0;
    for b in bits.iter_mut() {
        let (lo, hi) = mul_part(*b, m, overflow);
        *b = lo;
        overflow = hi;
    }
    overflow
}

fn mul_part(left: u32, right: u32, high: u32) -> (u32, u32) {
    let result = u64::from(left) * u64::from(right) + u64::from(high);
    let hi = ((result >> 32) & 0xFFFF_FFFF) as u32;
    let lo = (result & 0xFFFF_FFFF) as u32;
    (lo, hi)
}

fn div_internal(working: &mut [u32; 8], divisor: &[u32; 3]) {
    // There are a couple of ways to do division on binary numbers:
    //   1. Using long division
    //   2. Using the complement method
    // ref: https://www.wikihow.com/Divide-Binary-Numbers
    // The complement method basically keeps trying to subtract the
    // divisor until it can't anymore and placing the rest in remainder.
    let mut sub = [
        0u32,
        0u32,
        0u32,
        0u32,
        divisor[0] ^ 0xFFFF_FFFF,
        divisor[1] ^ 0xFFFF_FFFF,
        divisor[2] ^ 0xFFFF_FFFF,
        0xFFFF_FFFF,
    ];

    // Add one onto the complement, also, make sure remainder is 0
    let mut carry = 0;
    let one = [1u32, 0u32, 0u32, 0u32];
    for i in 4..8 {
        let sum = u64::from(sub[i]) + u64::from(one[i - 4]) + carry as u64;
        sub[i] = (sum & 0xFFFF_FFFF) as u32;
        carry = sum >> 32;

        // Zero out remainder at same time
        working[i] = 0;
    }

    // If we have nothing in our hi+ block then shift over till we do
    let mut blocks_to_process = 0;
    loop {
        if blocks_to_process >= 4 || working[3] != 0 {
            break;
        }
        // Shift whole blocks to the "left"
        working[3] = working[2];
        working[2] = working[1];
        working[1] = working[0];
        working[0] = 0;

        // Incremember the counter
        blocks_to_process += 1;
    }

    // Let's try and do the addition...
    let mut i = blocks_to_process << 5;
    loop {
        if i >= 128 {
            break;
        }

        // << 1 for the entire working array
        let mut shifted = 0;
        for i in 0..8 {
            let b = working[i] >> 31;
            working[i] = (working[i] << 1) | shifted;
            shifted = b;
        }

        // Copy the remainder of working into sub
        for j in 0..4 {
            sub[j] = working[j + 4];
        }

        // A little weird but we add together sub
        let mut carry = 0;
        for i in 0..4 {
            let sum = u64::from(sub[i]) + u64::from(sub[i + 4]) + carry as u64;
            sub[i] = (sum & 0xFFFF_FFFF) as u32;
            carry = sum >> 32;
        }

        // Was it positive?
        if (sub[3] & 0x80000000) == 0 {
            for j in 0..4 {
                working[j + 4] = sub[j];
            }
            working[0] |= 1;
        }

        // Increment our pointer
        i += 1;
    }
}

// Returns remainder
fn div_by_u32(bits: &mut [u32], divisor: u32) -> u32 {
    if divisor == 0 {
        // Divide by zero
        panic!("Internal error: divide by zero");
    } else if divisor == 1 {
        // dividend remains unchanged
        0
    } else {
        let mut remainder = 0u32;
        let divisor = u64::from(divisor);
        for i in (0..bits.len()).rev() {
            let temp = (u64::from(remainder) << 32) + u64::from(bits[i]);
            remainder = (temp % divisor) as u32;
            bits[i] = (temp / divisor) as u32;
        }

        remainder
    }
}

fn shl_internal(bits: &mut [u32; 3], shift: u32) {

    let mut shift = shift;

    // Whole blocks first
    while shift >= 32 {
        bits[2] = bits[1];
        bits[1] = bits[0];
        bits[0] = 0;
        shift -= 32;
    }

    // Continue with the rest
    if shift > 0 {
        let mut shifted = 0;
        for i in 0..3 {
            let b = bits[i] >> (32 - shift);
            bits[i] = (bits[i] << shift) | shifted;
            shifted = b;
        }
    }
}

#[inline]
fn cmp_internal(left: &[u32; 3], right: &[u32; 3]) -> Ordering {
    let left_hi: u32 = left[2];
    let right_hi: u32 = right[2];
    let left_lo: u64 = u64::from(left[1]) << 32 | u64::from(left[0]);
    let right_lo: u64 = u64::from(right[1]) << 32 | u64::from(right[0]);
    if left_hi < right_hi || (left_hi <= right_hi && left_lo < right_lo) {
        Ordering::Less
    } else if left_hi == right_hi && left_lo == right_lo {
        Ordering::Equal
    } else {
        Ordering::Greater
    }
}

fn is_all_zero(bits: &[u32]) -> bool {
    for b in bits.iter() {
        if *b != 0 {
            return false;
        }
    }
    true
}

fn is_some_zero(bits: &[u32], skip: usize, take: usize) -> bool {
    for b in bits.iter().skip(skip).take(take) {
        if *b != 0 {
            return false;
        }
    }
    true
}


macro_rules! impl_from {
    ($T:ty, $from_ty:path) => {
        impl From<$T> for Decimal {
            #[inline]
            fn from(t: $T) -> Decimal {
                $from_ty(t).unwrap()
            }
        }
    }
}

impl_from!(isize, FromPrimitive::from_isize);
impl_from!(i8, FromPrimitive::from_i8);
impl_from!(i16, FromPrimitive::from_i16);
impl_from!(i32, FromPrimitive::from_i32);
impl_from!(i64, FromPrimitive::from_i64);
impl_from!(usize, FromPrimitive::from_usize);
impl_from!(u8, FromPrimitive::from_u8);
impl_from!(u16, FromPrimitive::from_u16);
impl_from!(u32, FromPrimitive::from_u32);
impl_from!(u64, FromPrimitive::from_u64);

macro_rules! forward_val_val_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self).$method(&other)
            }
        }
    }
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl<'a> $imp<$res> for &'a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                self.$method(&other)
            }
        }
    }
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl<'a> $imp<&'a $res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                (&self).$method(other)
            }
        }
    }
}

macro_rules! forward_all_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        forward_val_val_binop!(impl $imp for $res, $method);
        forward_ref_val_binop!(impl $imp for $res, $method);
        forward_val_ref_binop!(impl $imp for $res, $method);
    };
}

impl Zero for Decimal {
    fn is_zero(&self) -> bool {
        self.lo.is_zero() && self.mid.is_zero() && self.hi.is_zero()
    }

    fn zero() -> Decimal {
        Decimal {
            flags: 0,
            hi: 0,
            lo: 0,
            mid: 0,
        }
    }
}

impl One for Decimal {
    fn one() -> Decimal {
        Decimal {
            flags: 0,
            hi: 0,
            lo: 1,
            mid: 0,
        }
    }
}

impl FromStr for Decimal {
    type Err = Error;

    fn from_str(value: &str) -> Result<Decimal, Self::Err> {
        if value.is_empty() {
            return Err(Error::new("Invalid decimal: empty"));
        }

        let mut offset = 0;
        let mut len = value.len();
        let chars: Vec<char> = value.chars().collect();
        let mut negative = false; // assume positive

        // handle the sign
        if chars[offset] == '-' {
            negative = true; // leading minus means negative
            offset += 1;
            len -= 1;
        } else if chars[offset] == '+' {
            // leading + allowed
            offset += 1;
            len -= 1;
        }

        // should now be at numeric part of the significand
        let mut dot_offset: i32 = -1; // '.' offset, -1 if none
        let cfirst = offset; // record start of integer
        let mut coeff = String::new(); // integer significand array

        while len > 0 {
            let c = chars[offset];
            if c.is_digit(10) {
                coeff.push(c);
                offset += 1;
                len -= 1;
                continue;
            }
            if c == '.' {
                if dot_offset >= 0 {
                    return Err(Error::new("Invalid decimal: two decimal points"));
                }
                dot_offset = offset as i32;
                offset += 1;
                len -= 1;
                continue;
            }

            return Err(Error::new("Invalid decimal: unknown character"));
        }

        // here when no characters left
        if coeff.is_empty() {
            return Err(Error::new("Invalid decimal: no digits found"));
        }

        let mut scale = 0u32;
        if dot_offset >= 0 {
            // we had a decimal place so set the scale
            scale = (coeff.len() as u32) - (dot_offset as u32 - cfirst as u32);
        }

        // Parse this into a big uint
        let res = BigUint::from_str(&coeff[..]);
        if res.is_err() {
            return Err(Error::new("Failed to parse string"));
        }

        Decimal::from_biguint(res.unwrap(), scale, negative)
    }
}

impl FromPrimitive for Decimal {
    fn from_i32(n: i32) -> Option<Decimal> {
        let flags: u32;
        let value_copy: i32;
        if n >= 0 {
            flags = 0;
            value_copy = n;
        } else {
            flags = SIGN_MASK;
            value_copy = -n;
        }
        Some(Decimal {
            flags: flags,
            lo: value_copy as u32,
            mid: 0,
            hi: 0,
        })
    }

    fn from_i64(n: i64) -> Option<Decimal> {
        let flags: u32;
        let value_copy: i64;
        if n >= 0 {
            flags = 0;
            value_copy = n;
        } else {
            flags = SIGN_MASK;
            value_copy = -n;
        }
        Some(Decimal {
            flags: flags,
            lo: value_copy as u32,
            mid: (value_copy >> 32) as u32,
            hi: 0,
        })
    }

    fn from_u32(n: u32) -> Option<Decimal> {
        Some(Decimal {
            flags: 0,
            lo: n,
            mid: 0,
            hi: 0,
        })
    }

    fn from_u64(n: u64) -> Option<Decimal> {
        Some(Decimal {
            flags: 0,
            lo: n as u32,
            mid: (n >> 32) as u32,
            hi: 0,
        })
    }

    fn from_f32(n: f32) -> Option<Decimal> {
        // Handle the case if it is NaN, Infinity or -Infinity
        if !n.is_finite() {
            return None;
        }

        // It's a shame we can't use a union for this due to it being broken up by bits
        // i.e. 1/8/23 (sign, exponent, mantissa)
        // See https://en.wikipedia.org/wiki/IEEE_754-1985
        // n = (sign*-1) * 2^exp * mantissa
        // Decimal of course stores this differently... 10^-exp * significand
        let raw = n.to_bits();
        let positive = (raw >> 31) == 0;
        let biased_exponent = ((raw >> 23) & 0xFF) as i32;
        let mantissa = raw & 0x007F_FFFF;

        // Handle the special zero case
        if biased_exponent == 0 && mantissa == 0 {
            let mut zero = Decimal::zero();
            if !positive {
                zero.set_sign(false);
            }
            return Some(zero);
        }

        // Get the bits and exponent2
        let mut exponent2 = biased_exponent - 127;
        let mut bits = [mantissa, 0u32, 0u32];
        if biased_exponent == 0 {
            // Denormalized number - correct the exponent
            exponent2 += 1;
        } else {
            // Add extra hidden bit to mantissa
            bits[0] |= 0x0080_0000;
        }

        // The act of copying a mantissa as integer bits is equivalent to shifting
        // left the mantissa 23 bits. The exponent is reduced to compensate.
        exponent2 -= 23;

        // Convert to decimal
        Decimal::base2_to_decimal(&mut bits, exponent2, positive, false)
    }

    fn from_f64(n: f64) -> Option<Decimal> {
        // Handle the case if it is NaN, Infinity or -Infinity
        if !n.is_finite() {
            return None;
        }

        // It's a shame we can't use a union for this due to it being broken up by bits
        // i.e. 1/11/52 (sign, exponent, mantissa)
        // See https://en.wikipedia.org/wiki/IEEE_754-1985
        // n = (sign*-1) * 2^exp * mantissa
        // Decimal of course stores this differently... 10^-exp * significand
        let raw = n.to_bits();
        let positive = (raw >> 63) == 0;
        let biased_exponent = ((raw >> 52) & 0x7FF) as i32;
        let mantissa = raw & 0x000F_FFFF_FFFF_FFFF;

        // Handle the special zero case
        if biased_exponent == 0 && mantissa == 0 {
            let mut zero = Decimal::zero();
            if !positive {
                zero.set_sign(false);
            }
            return Some(zero);
        }

        // Get the bits and exponent2
        let mut exponent2 = biased_exponent - 1023;
        let mut bits = [
            (mantissa & 0xFFFF_FFFF) as u32,
            ((mantissa >> 32) & 0xFFFF_FFFF) as u32,
            0u32,
        ];
        if biased_exponent == 0 {
            // Denormalized number - correct the exponent
            exponent2 += 1;
        } else {
            // Add extra hidden bit to mantissa
            bits[1] |= 0x0010_0000;
        }

        // The act of copying a mantissa as integer bits is equivalent to shifting
        // left the mantissa 52 bits. The exponent is reduced to compensate.
        exponent2 -= 52;

        // Convert to decimal
        Decimal::base2_to_decimal(&mut bits, exponent2, positive, true)
    }
}

impl ToPrimitive for Decimal {
    fn to_f64(&self) -> Option<f64> {
        if self.scale() == 0 {
            let bytes = self.unsigned_bytes_le();
            let sign;
            if self.is_negative() {
                sign = Minus;
            } else {
                sign = Plus;
            }

            BigInt::from_bytes_le(sign, &bytes[..]).to_f64()
        } else {
            match self.to_string().parse::<f64>() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    }

    fn to_i64(&self) -> Option<i64> {
        let d = self.rescale(0);
        // Convert to biguint and use that
        let bytes = d.unsigned_bytes_le();
        let sign;
        if self.is_negative() {
            sign = Minus;
        } else {
            sign = Plus;
        }
        BigInt::from_bytes_le(sign, &bytes[..]).to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        if self.is_negative() {
            return None;
        }

        // Rescale to 0 (truncate)
        let d = self.rescale(0);
        if d.hi != 0 {
            // Overflow
            return None;
        }

        // Convert to biguint and use that
        let bytes = d.unsigned_bytes_le();
        BigUint::from_bytes_le(&bytes[..]).to_u64()
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Get the scale - where we need to put the decimal point
        let mut scale = self.scale() as usize;

        // Get the whole number without decimal points (or signs)
        let uint = self.to_biguint();

        // Convert to a string and manipulate that (neg at front, inject decimal)
        let mut rep = uint.to_string();
        let len = rep.len();


        if let Some(n_dp) = f.precision() {
            if n_dp < scale {
                rep.truncate(len - scale + n_dp)
            } else {
                let zeros = repeat("0").take(n_dp - scale).collect::<String>();
                rep.push_str(&zeros[..]);
            }
            scale = n_dp;
        }
        let len = rep.len();

        // Inject the decimal point
        if scale > 0 {
            // Must be a low fractional
            if scale > len {
                let mut new_rep = String::new();
                let zeros = repeat("0").take(scale as usize - len).collect::<String>();
                new_rep.push_str("0.");
                new_rep.push_str(&zeros[..]);
                new_rep.push_str(&rep[..]);
                rep = new_rep;
            } else if scale == len {
                rep.insert(0, '.');
                rep.insert(0, '0');
            } else {
                rep.insert(len - scale as usize, '.');
            }
        } else if rep.is_empty() {
            // corner case for when we truncated everything in a low fractional
            rep.insert(0, '0');
        }

        f.pad_integral(self.is_positive(), "", &rep)
    }
}

forward_all_binop!(impl Add for Decimal, add);

impl<'a, 'b> Add<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn add(self, other: &Decimal) -> Decimal {

        // Convert to the same scale
        let my_scale = self.scale();
        let other_scale = other.scale();
        let mut flags;
        let mut my = [self.lo, self.mid, self.hi];
        let mut ot = [other.lo, other.mid, other.hi];
        if my_scale > other_scale {
            let rescaled = other.rescale(my_scale);
            ot[0] = rescaled.lo;
            ot[1] = rescaled.mid;
            ot[2] = rescaled.hi;
            flags = my_scale << SCALE_SHIFT;
        } else if other_scale > my_scale {
            let rescaled = self.rescale(other_scale);
            my[0] = rescaled.lo;
            my[1] = rescaled.mid;
            my[2] = rescaled.hi;
            flags = other_scale << SCALE_SHIFT;
        } else {
            flags = my_scale << SCALE_SHIFT;
        }

        // Add the items together
        let my_negative = self.is_negative();
        let other_negative = other.is_negative();
        if my_negative && other_negative {
            flags |= SIGN_MASK;
            add_internal(&mut my, &ot);
        } else if my_negative && !other_negative {
            // -x + y
            let cmp = cmp_internal(&my, &ot);
            // if x > y then it's negative (i.e. -2 + 1)
            match cmp {
                Ordering::Less => {
                    sub_internal(&mut ot, &my);
                    my[0] = ot[0];
                    my[1] = ot[1];
                    my[2] = ot[2];
                }
                Ordering::Greater => {
                    flags |= SIGN_MASK;
                    sub_internal(&mut my, &ot);
                }
                Ordering::Equal => {
                    // -2 + 2
                    my[0] = 0;
                    my[1] = 0;
                    my[2] = 0;
                }
            }
        } else if !my_negative && other_negative {
            // x + -y
            let cmp = cmp_internal(&my, &ot);
            // if x < y then it's negative (i.e. 1 + -2)
            match cmp {
                Ordering::Less => {
                    flags |= SIGN_MASK;
                    sub_internal(&mut ot, &my);
                    my[0] = ot[0];
                    my[1] = ot[1];
                    my[2] = ot[2];
                }
                Ordering::Greater => {
                    sub_internal(&mut my, &ot);
                }
                Ordering::Equal => {
                    // 2 + -2
                    my[0] = 0;
                    my[1] = 0;
                    my[2] = 0;
                }
            }
        } else {
            add_internal(&mut my, &ot);
        }
        Decimal {
            lo: my[0],
            mid: my[1],
            hi: my[2],
            flags: flags,
        }
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, other: Decimal) {
        let result = self.add(other);
        self.lo = result.lo;
        self.mid = result.mid;
        self.hi = result.hi;
        self.flags = result.flags;
    }
}

forward_all_binop!(impl Sub for Decimal, sub);

impl<'a, 'b> Sub<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn sub(self, other: &Decimal) -> Decimal {
        let negated_other = Decimal {
            lo: other.lo,
            mid: other.mid,
            hi: other.hi,
            flags: other.flags ^ SIGN_MASK,
        };
        self.add(negated_other)
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, other: Decimal) {
        let result = self.sub(other);
        self.lo = result.lo;
        self.mid = result.mid;
        self.hi = result.hi;
        self.flags = result.flags;
    }
}

forward_all_binop!(impl Mul for Decimal, mul);

impl<'a, 'b> Mul<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn mul(self, other: &Decimal) -> Decimal {
        // Early exit if either is zero
        if self.is_zero() || other.is_zero() {
            return Decimal {
                lo: 0,
                mid: 0,
                hi: 0,
                flags: 0,
            };
        }

        let my = [self.lo, self.mid, self.hi];
        let ot = [other.lo, other.mid, other.hi];

        // Start a result array
        let mut result = [0u32, 0u32, 0u32];

        // We are only resulting in a negative if we have mismatched signs
        let negative = self.is_negative() ^ other.is_negative();

        // We get the scale of the result by adding the operands. This may be too big, however
        //  we'll correct later
        let my_scale = self.scale();
        let ot_scale = other.scale();
        let mut final_scale = my_scale + ot_scale;

        // Do the calculation, this first part is just trying to shortcut cycles.
        let to = if my[2] == 0 && my[1] == 0 {
            1
        } else if my[2] == 0 {
            2
        } else {
            3
        };
        // We calculate into a 256 bit number temporarily
        let mut running: [u32; 6] = [0, 0, 0, 0, 0, 0];
        let mut overflow = 0;
        for i in 0..to {
            for j in 0..3 {
                let (res, of) = mul_part(ot[j], my[i], overflow);
                overflow = of;
                let running_index = i + j;
                let mut working = res;
                loop {
                    let added = running[running_index] as u64 + working as u64;
                    running[running_index] = (added & 0xFFFF_FFFF) as u32;
                    working = (added >> 32) as u32;
                    if working == 0 {
                        break;
                    }
                }
            }
        }

        // While our result is in overflow (i.e. upper portion != 0)
        // AND it has a scale > 0 we divide by 10
        let mut remainder = 0;
        while final_scale > 0 && !is_some_zero(&running, 3, 3) {
            remainder = div_by_u32(&mut running, 10u32);
            final_scale -= 1;
        }

        // Round up the carry if we need to
        if remainder >= 5 {
            for i in 0..6 {
                let digit = running[i] as u64 + 1;
                running[i] = (digit & 0xFFFF_FFFF) as u32;
                if digit <= 0xFFFF_FFFF {
                    break;
                }
            }
        }

        // If our upper portion is not 0, we've overflowed
        if !(running[3] == 0 && running[4] == 0 && running[5] == 0) {
            panic!("Multiplication overflowed");
        }

        // Copy to our result
        result[0] = running[0];
        result[1] = running[1];
        result[2] = running[2];

        // We underflowed, we'll lose precision.
        // For now we panic however perhaps in the future I could give the option to round
        if final_scale > MAX_PRECISION {
            panic!("Multiplication underflowed");
        }

        // We have our result
        let flags = (final_scale << SCALE_SHIFT) | if negative { SIGN_MASK } else { 0 };
        Decimal {
            lo: result[0],
            mid: result[1],
            hi: result[2],
            flags: flags,
        }
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, other: Decimal) {
        let result = self.mul(other);
        self.lo = result.lo;
        self.mid = result.mid;
        self.hi = result.hi;
        self.flags = result.flags;
    }
}

forward_all_binop!(impl Div for Decimal, div);

impl<'a, 'b> Div<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn div(self, other: &Decimal) -> Decimal {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self.is_zero() {
            return Decimal::zero();
        }

        let dividend = [self.lo, self.mid, self.hi];
        let divisor = [other.lo, other.mid, other.hi];
        let dividend_scale = self.scale();
        let divisor_scale = other.scale();

        // Division is the most tricky...
        // 1. If it's the first iteration, we use the intended dividend.
        // 2. If the remainder != 0 from the previous iteration, we use it
        //    as the dividend for this iteration
        // 3. We use this to calculate the quotient and remainder
        // 4. We add this quotient to the final result
        // 5. We multiply the integer part of the remainder by 10 and up the
        //    scale to maintain precision.
        // 6. Loop back to step 2 until:
        //       a. the remainder is zero (i.e. 6/3 = 2) OR
        //       b. addition in 4 fails to modify bits in quotient (i.e. due to underflow)
        let mut quotient = [0u32, 0u32, 0u32];
        let mut quotient_scale: i32 = dividend_scale as i32 - divisor_scale as i32;

        // Working is the remainder + the quotient
        // We use an aligned array since we'll be using it alot.
        let mut working = [
            dividend[0],
            dividend[1],
            dividend[2],
            0u32,
            0u32,
            0u32,
            0u32,
            0u32,
        ];
        let mut working_scale = quotient_scale;
        let mut remainder_scale = working_scale;
        let mut underflow;

        loop {
            div_internal(&mut working, &divisor);
            underflow = add_with_scale_internal(
                &mut quotient,
                &mut quotient_scale,
                &mut working,
                &mut working_scale,
            );
            // TODO: We could round here however I don't want it to be lossy

            // Multiply the remainder by 10
            let mut overflow = 0;
            for i in 4..8 {
                let (lo, hi) = mul_part(working[i] as u32, 10, overflow);
                working[i] = lo;
                overflow = hi;
            }
            // Copy it into the quotient section
            for i in 0..4 {
                working[i] = working[i + 4];
            }

            remainder_scale += 1;
            working_scale = remainder_scale;

            if underflow || is_some_zero(&working, 4, 4) {
                break;
            }
        }

        // If we have a really big number try to adjust the scale to 0
        if !underflow {
            while quotient_scale < 0 {
                for i in 0..8 {
                    if i < 3 {
                        working[i] = quotient[i];
                    } else {
                        working[i] = 0;
                    }
                }

                // Mul 10
                let mut overflow = 0;
                for i in 0..8 {
                    let (lo, hi) = mul_part(working[i] as u32, 10, overflow);
                    working[i] = lo;
                    overflow = hi;
                }
                if is_some_zero(&working, 3, 5) {
                    quotient_scale += 1;
                    quotient[0] = working[0];
                    quotient[1] = working[1];
                    quotient[2] = working[2];
                } else {
                    // Overflow
                    panic!("Division overflowed");
                }
            }

            if quotient_scale > 255 {
                quotient[0] = 0;
                quotient[1] = 0;
                quotient[2] = 0;
                quotient_scale = 0;
            }
        }

        let quotient_negative = self.is_negative() ^ other.is_negative();
        Decimal {
            lo: quotient[0],
            mid: quotient[1],
            hi: quotient[2],
            flags: (quotient_scale << SCALE_SHIFT) as u32 | if quotient_negative { SIGN_MASK } else { 0 },
        }
    }
}

impl DivAssign for Decimal {
    fn div_assign(&mut self, other: Decimal) {
        let result = self.div(other);
        self.lo = result.lo;
        self.mid = result.mid;
        self.hi = result.hi;
        self.flags = result.flags;
    }
}

forward_all_binop!(impl Rem for Decimal, rem);

impl<'a, 'b> Rem<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn rem(self, other: &Decimal) -> Decimal {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self.is_zero() {
            return Decimal::zero();
        }


        // Working is the remainder + the quotient
        // We use an aligned array since we'll be using it alot.
        let mut working = [self.lo, self.mid, self.hi, 0u32, 0u32, 0u32, 0u32, 0u32];
        let divisor = [other.lo, other.mid, other.hi];
        div_internal(&mut working, &divisor);

        // Remainder has no scale however does have a sign (the same as self)
        Decimal {
            lo: working[4],
            mid: working[5],
            hi: working[6],
            flags: if self.is_negative() { SIGN_MASK } else { 0 },
        }
    }
}

impl RemAssign for Decimal {
    fn rem_assign(&mut self, other: Decimal) {
        let result = self.rem(other);
        self.lo = result.lo;
        self.mid = result.mid;
        self.hi = result.hi;
        self.flags = result.flags;
    }
}

impl PartialEq for Decimal {
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        self.cmp(other) == Equal
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Decimal) -> Ordering {
        // Quick exit if major differences
        let self_negative = self.is_negative();
        let other_negative = other.is_negative();
        if self_negative && !other_negative {
            return Ordering::Less;
        } else if !self_negative && other_negative {
            return Ordering::Greater;
        }

        // If we have 1.23 and 1.2345 then we have
        //  123 scale 2 and 12345 scale 4
        //  We need to convert the first to
        //  12300 scale 4 so we can compare equally
        let s = self.scale() as u32;
        let o = other.scale() as u32;

        if s == o {
            // Fast path for same scale
            if self.hi != other.hi {
                return self.hi.cmp(&other.hi);
            }
            if self.mid != other.mid {
                return self.mid.cmp(&other.mid);
            }
            return self.lo.cmp(&other.lo);
        }

        let si = self.to_bigint();
        let oi = other.to_bigint();
        if s > o {
            let power = power_10((s - o) as usize).to_bigint().unwrap();
            let other_scaled = oi * power;
            si.cmp(&other_scaled)
        } else {
            let power = power_10((o - s) as usize).to_bigint().unwrap();
            let self_scaled = si * power;
            self_scaled.cmp(&oi)
        }
    }
}


#[cfg(test)]
mod test {
    // Tests on private methods.
    //
    // All public tests should go under `tests/`.

    use super::*;
    #[test]
    fn rescale_integer_up() {
        for scale in 1..25 {
            let d = "1".parse::<Decimal>().unwrap().rescale(scale);

            let mut s = String::from("1.");
            for _ in 0..scale {
                s.push('0');
            }

            assert_eq!(d.to_string(), s);
        }
    }

    #[test]
    fn rescale_integer_down() {
        for scale in 1..25 {
            let d = "1.000000000000000000000000"
                .parse::<Decimal>()
                .unwrap()
                .rescale(scale);

            let mut s = String::from("1.");
            for _ in 0..scale {
                s.push('0');
            }

            assert_eq!(d.to_string(), s);
        }
    }

    #[test]
    fn rescale_float_up() {
        for scale in 1..25 {
            let d = "1.1".parse::<Decimal>().unwrap().rescale(scale);

            let mut s = String::from("1.1");
            for _ in 0..(scale - 1) {
                s.push('0');
            }

            assert_eq!(d.to_string(), s);
        }
    }

    #[test]
    fn rescale_float_down() {
        for scale in 1..24 {
            let d = "1.000000000000000000000001"
                .parse::<Decimal>()
                .unwrap()
                .rescale(scale);

            let mut s = String::from("1.");
            for _ in 0..(scale) {
                s.push('0');
            }

            assert_eq!(d.to_string(), s);
        }
    }

    #[test]
    fn round_complex_number() {
        // This is 1982.2708333333333
        let a = Decimal {
            flags: 1572864,
            hi: 107459117,
            lo: 2219136341,
            mid: 849254895,
        };
        let b = a.round_dp(2u32);
        assert_eq!("1982.27", b.to_string());
    }
}
