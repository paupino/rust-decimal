use Error;
use num::{BigInt, BigUint, FromPrimitive, Integer, One, ToPrimitive, Zero};
use num::bigint::Sign::{Minus, Plus};
use std::cmp::*;
use std::cmp::Ordering::Equal;
use std::fmt;
use std::iter::repeat;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

// Sign mask for the flags field. A value of zero in this bit indicates a
// positive Decimal value, and a value of one in this bit indicates a
// negative Decimal value.
#[allow(overflowing_literals)]
const SIGN_MASK: i32 = 0x80000000;

// Scale mask for the flags field. This byte in the flags field contains
// the power of 10 to divide the Decimal value by. The scale byte must
// contain a value between 0 and 28 inclusive.
const SCALE_MASK: i32 = 0x00FF0000;
const U8_MASK: i32 = 0x000000FF;
const I32_MASK: i64 = 0xFFFFFFFF;

// Number of bits scale is shifted by.
const SCALE_SHIFT: i32 = 16;

// The maximum supported precision
const MAX_PRECISION: u32 = 28;
const MAX_BYTES: usize = 12;
const MAX_BITS: usize = 96;

lazy_static! {
    static ref MIN: Decimal = Decimal { flags: -2147483648, lo: -1, mid: -1, hi: -1 };
    static ref MAX: Decimal = Decimal { flags: 0, lo: -1, mid: -1, hi: -1 };
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
    flags: i32,
    // The lo, mid, hi, and flags fields contain the representation of the
    // Decimal value as a 96-bit integer.
    hi: i32,
    lo: i32,
    mid: i32,
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
            panic!("Scale exceeds the maximum precision allowed");
        }
        let flags: i32 = (scale as i32) << SCALE_SHIFT;
        if num < 0 {
            return Decimal {
                flags: flags | SIGN_MASK,
                hi: 0,
                lo: (num.abs() & I32_MASK) as i32,
                mid: ((num.abs() >> 32) & I32_MASK) as i32,
            };
        }
        Decimal {
            flags: flags,
            hi: 0,
            lo: (num & I32_MASK) as i32,
            mid: ((num >> 32) & I32_MASK) as i32,
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
            flags: (bytes[0] as i32) | (bytes[1] as i32) << 8 | (bytes[2] as i32) << 16 | (bytes[3] as i32) << 24,
            lo: (bytes[4] as i32) | (bytes[5] as i32) << 8 | (bytes[6] as i32) << 16 | (bytes[7] as i32) << 24,
            mid: (bytes[8] as i32) | (bytes[9] as i32) << 8 | (bytes[10] as i32) << 16 | (bytes[11] as i32) << 24,
            hi: (bytes[12] as i32) | (bytes[13] as i32) << 8 | (bytes[14] as i32) << 16 | (bytes[15] as i32) << 24,
        }
    }

    /// Returns `true` if the decimal is negative.
    pub fn is_negative(&self) -> bool {
        self.flags < 0
    }

    /// Returns `true` if the decimal is positive.
    pub fn is_positive(&self) -> bool {
        self.flags >= 0
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

        if dp < old_scale && dp < 20 {
            // Technically, it's 28...
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
            //println!("{} * {}", self.to_string(), power10.to_string());
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
            //println!("Raw: {}, Offset: {}", raw.to_string(), offset.to_string());
            let decimal_portion = raw - offset;

            // Rescale to zero so it's easier to work with
            value = value.rescale(0u32);

            // If the decimal_portion is zero then we round based on the other data
            let mut cap = BigUint::from_u32(5u32).unwrap();
            for _ in 0..(old_scale - dp - 1) {
                cap = cap.mul(BigUint::from_u32(10u32).unwrap());
            }
            //println!("Cap {} Decimal Portion {}", cap, decimal_portion);
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
                    //println!("Decimal is greater than cap {} > {}", decimal_portion, cap);
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
        let index = diff.abs() as usize;
        if diff > 0 {
            if index < 10 {
                result = unsigned * BigUint::from_u32(POWERS_10[index]).unwrap();
            } else if index < 20 {
                result = unsigned * BigUint::from_u64(BIG_POWERS_10[index - 10]).unwrap();
            } else {
                let u32_index = index - 19; // -20 + 1 for getting the right u32 index
                let exponent = BigUint::from_u64(BIG_POWERS_10[9]).unwrap() *
                    BigUint::from_u32(POWERS_10[u32_index]).unwrap();
                result = unsigned * exponent;
            }
        } else {
            if index < 10 {
                result = unsigned / BigUint::from_u32(POWERS_10[index]).unwrap();
            } else if index < 20 {
                result = unsigned / BigUint::from_u64(BIG_POWERS_10[index - 10]).unwrap();
            } else {
                let u32_index = index - 19; // -20 + 1 for getting the right u32 index
                let exponent = BigUint::from_u64(BIG_POWERS_10[9]).unwrap() *
                    BigUint::from_u32(POWERS_10[u32_index]).unwrap();
                result = unsigned / exponent;
            }
        }

        // Convert it back
        let bytes = result.to_bytes_le();
        Decimal::from_bytes_le(bytes, exp, self.is_negative())
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
        let mut flags = 0i32;
        let mut lo = 0i32;
        let mut mid = 0i32;
        let mut hi = 0i32;

        if scale > 0 {
            flags = (scale as i32) << SCALE_SHIFT;
        }
        if negative {
            flags |= SIGN_MASK;
        }
        if bytes.len() > MAX_BYTES {
            panic!("Decimal Overflow");
        }

        let mut pos = 0;
        for b in bytes {
            if pos < 4 {
                lo |= (b as i32) << (pos * 8);
            } else if pos < 8 {
                mid |= (b as i32) << ((pos - 4) * 8);
            } else {
                hi |= (b as i32) << ((pos - 8) * 8);
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

        // println!("coeff.len() {}, dot_offset {} cfirst {} negative {}", coeff.len(), dot_offset, cfirst, negative);
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
        let flags: i32;
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
            lo: value_copy,
            mid: 0,
            hi: 0,
        })
    }

    fn from_i64(n: i64) -> Option<Decimal> {
        let flags: i32;
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
            lo: value_copy as i32,
            mid: (value_copy >> 32) as i32,
            hi: 0,
        })
    }

    fn from_u32(n: u32) -> Option<Decimal> {
        Some(Decimal {
            flags: 0,
            lo: n as i32,
            mid: 0,
            hi: 0,
        })
    }

    fn from_u64(n: u64) -> Option<Decimal> {
        Some(Decimal {
            flags: 0,
            lo: n as i32,
            mid: (n >> 32) as i32,
            hi: 0,
        })
    }

    fn from_f64(n: f64) -> Option<Decimal> {
        // Handle the case if it is NaN, Infinity or -Infinity
        if !n.is_finite() {
            return None;
        }

        None
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
                Err(_) => None
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

fn scaled_biguints(me: &Decimal, other: &Decimal) -> (BigUint, BigUint, u32) {
    // Scale to the max
    let s_scale = me.scale();
    let o_scale = other.scale();

    if s_scale > o_scale {
        (
            me.to_biguint(),
            other.rescale(s_scale).to_biguint(),
            s_scale,
        )
    } else if o_scale > s_scale {
        (
            me.rescale(o_scale).to_biguint(),
            other.to_biguint(),
            o_scale,
        )
    } else {
        (me.to_biguint(), other.to_biguint(), s_scale)
    }
}

fn scaled_bigints(me: &Decimal, other: &Decimal) -> (BigInt, BigInt, u32) {
    // Scale to the max
    let s_scale = me.scale();
    let o_scale = other.scale();

    if s_scale > o_scale {
        (me.to_bigint(), other.rescale(s_scale).to_bigint(), s_scale)
    } else if o_scale > s_scale {
        (me.rescale(o_scale).to_bigint(), other.to_bigint(), o_scale)
    } else {
        (me.to_bigint(), other.to_bigint(), s_scale)
    }
}

forward_all_binop!(impl Add for Decimal, add);

impl<'a, 'b> Add<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn add(self, other: &Decimal) -> Decimal {

        // Get big uints to work with
        let (left, right, scale) = scaled_biguints(self, other);

        // Now we have the big boys - do a quick add
        // println!("Left {} Right {}", left, right);
        let l_negative = self.is_negative();
        let r_negative = other.is_negative();
        let result;
        let is_negative;
        if l_negative && r_negative {
            result = left + right;
            is_negative = true;
        } else if !l_negative && !r_negative {
            result = left + right;
            is_negative = false;
        } else {
            //  1 + -2 (l < r, -r => r - l, -)
            //  2 + -1 (l > r, -r => l - r, +)
            // -1 +  2 (l < r, -l => r - l, +)
            // -2 +  1 (l > r, -l => l - r, -)
            if r_negative {
                if left < right {
                    result = right - left;
                    is_negative = true;
                } else if left > right {
                    result = left - right;
                    is_negative = false;
                } else {
                    result = BigUint::zero();
                    is_negative = false;
                }
            } else {
                // l_negative
                if left < right {
                    result = right - left;
                    is_negative = false;
                } else if left > right {
                    result = left - right;
                    is_negative = true;
                } else {
                    result = BigUint::zero();
                    is_negative = false;
                }
            }
        }

        // Convert it back
        let bytes = result.to_bytes_le();
        Decimal::from_bytes_le(bytes, scale, is_negative)
    }
}

forward_all_binop!(impl Sub for Decimal, sub);

impl<'a, 'b> Sub<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn sub(self, other: &Decimal) -> Decimal {
        // Get big uints to work with
        let (left, right, scale) = scaled_biguints(self, other);

        // Now we have the big boys - do a quick subtraction
        // Both Positive:
        // 1 - 2 = -1
        // 2 - 1 = 1
        // Both negative:
        // -1 - -2 = 1
        // -2 - -1 = -1
        // Mismatch
        // -1 - 2 = -3
        // -2 - 1 = -3
        // 1 - -2 = 3
        // 2 - -1 = 3
        let l_negative = self.is_negative();
        let r_negative = other.is_negative();
        let result: BigUint;
        let is_negative: bool;
        if l_negative ^ r_negative {
            result = left + right;
            is_negative = l_negative;
        } else {
            if left > right {
                result = left - right;
                is_negative = l_negative && r_negative;
            } else {
                result = right - left;
                is_negative = !l_negative && !r_negative;
            }
        }

        // Convert it back
        let bytes = result.to_bytes_le();
        Decimal::from_bytes_le(bytes, scale, is_negative && !result.is_zero())
    }
}

forward_all_binop!(impl Mul for Decimal, mul);

impl<'a, 'b> Mul<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn mul(self, other: &Decimal) -> Decimal {
        // Get big uints to work with
        let left = self.to_biguint();
        let right = other.to_biguint();

        // Easy!
        let mut result = left * right; // Has the potential to overflow below if > 2^96
        let mut scale = self.scale() + other.scale();
        //println!("Result: {}, Scale: {}", result, scale);
        //println!("Self Scale: {}, Other Scale: {}", self.scale(), other.scale());

        // The result may be an overflow of what we can comfortably represent in 96 bits
        // We can only do this if we have a scale to work with
        if result.bits() > MAX_BITS {
            // Try to truncate until we're ok
            let ten = BigUint::from_i32(10).unwrap();
            while scale > 0 && result.bits() > 96 {
                result = result / &ten;
                scale -= 1;
                //println!("result: {} new scale: {}", result, scale);
            }
        }

        // Last check for overflow
        if result.bits() > MAX_BITS {
            panic!("Decimal overflow from multiplication");
        }

        if scale > MAX_PRECISION {
            // Then what? Truncate?
            panic!("Scale overflow; cannot represent exp {}", scale);
        }
        // Negativity is based on xor. e.g.
        // 1 * 2 = 2
        // -1 * 2 = -2
        // 1 * -2 = -2
        // -1 * -2 = 2
        let bytes = result.to_bytes_le();
        Decimal::from_bytes_le(bytes, scale, self.is_negative() ^ other.is_negative())
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
        // Shortcircuit the basic cases
        if self.is_zero() {
            return Decimal::zero();
        }

        let mut rem: BigUint;
        let ten = BigUint::from_i32(10).unwrap();
        let mut fractional: Vec<u8> = Vec::new();

        // Get the values
        let (left, right, _) = scaled_biguints(self, other);

        // The algorithm for this is:
        //  (integral, rem) = div_rem(x, y)
        //  while rem > 0 {
        //      (part, rem) = div_rem(rem * 10, y)
        //      fractional_part.push(part)
        //  }
        // This could be a really big number.
        //  Consider 9,999,999,999,999/10,000,000,000,000
        //  This would be (0, 9,999,999,999,999)
        let (i, r) = left.div_rem(&right);
        let mut integral = i;
        let length = if integral.is_zero() {
            0usize
        } else {
            integral.to_string().len()
        };
        rem = r;

        // This is slightly too agressive. But it is just being safe. We need to check against Decimal::MAX
        while !rem.is_zero() && fractional.len() + length < MAX_PRECISION as usize {
            let rem_carried = &ten * rem;
            let (frac, r) = rem_carried.div_rem(&right);
            fractional.push(frac.to_u8().unwrap());
            rem = r;
        }

        // Add on the fractional part
        let scale = fractional.len();
        for f in fractional {
            integral = integral * &ten + BigUint::from_u8(f).unwrap();
        }

        let bytes = integral.to_bytes_le();
        // Negative only if one or the other is negative
        Decimal::from_bytes_le(
            bytes,
            scale as u32,
            self.is_negative() ^ other.is_negative(),
        )
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

        // Shortcircuit the basic case
        if self.is_zero() {
            return Decimal::zero();
        }

        // Make sure they're scaled
        let (left, right, scale) = scaled_bigints(self, other);
        //println!("{}, {}", left, right);

        // Since we're just getting the remainder, we simply need to do a standard mod
        let (_, remainder) = left.div_rem(&right);

        // Remainder is always positive?
        let (sign, bytes) = remainder.to_bytes_le();
        Decimal::from_bytes_le(bytes, scale, sign == Minus)
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
        // If we have 1.23 and 1.2345 then we have
        //  123 scale 2 and 12345 scale 4
        //  We need to convert the first to
        //  12300 scale 4 so we can compare equally
        let s = self.scale() as u32;
        let o = other.scale() as u32;
        if s > o {
            let d = other.rescale(s);
            return self.cmp(&d);
        } else if s < o {
            let d = self.rescale(o);
            return (&d).cmp(other);
        }

        // Convert to big int
        let si = self.to_bigint();
        let oi = other.to_bigint();
        si.cmp(&oi)
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
            lo: -2075830955,
            mid: 849254895,
        };
        let b = a.round_dp(2u32);
        assert_eq!("1982.27", b.to_string());
    }
}
