use crate::Error;
use alloc::{string::String, vec::Vec};
use arrayvec::{ArrayString, ArrayVec};
use core::{
    cmp::{Ordering::Equal, *},
    fmt,
    hash::{Hash, Hasher},
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
    str::FromStr,
};
#[cfg(feature = "diesel")]
use diesel::sql_types::Numeric;
#[allow(unused_imports)] // It's not actually dead code below, but the compiler thinks it is.
#[cfg(not(feature = "std"))]
use num_traits::float::FloatCore;
use num_traits::{FromPrimitive, Num, One, Signed, ToPrimitive, Zero};

// Sign mask for the flags field. A value of zero in this bit indicates a
// positive Decimal value, and a value of one in this bit indicates a
// negative Decimal value.
const SIGN_MASK: u32 = 0x8000_0000;
const UNSIGN_MASK: u32 = 0x4FFF_FFFF;

// Scale mask for the flags field. This byte in the flags field contains
// the power of 10 to divide the Decimal value by. The scale byte must
// contain a value between 0 and 28 inclusive.
const SCALE_MASK: u32 = 0x00FF_0000;
const U8_MASK: u32 = 0x0000_00FF;
const U32_MASK: u64 = 0xFFFF_FFFF;

// Number of bits scale is shifted by.
const SCALE_SHIFT: u32 = 16;
// Number of bits sign is shifted by.
const SIGN_SHIFT: u32 = 31;

// The maximum string buffer size used for serialization purposes. 31 is optimal, however we align
// to the byte boundary for simplicity.
const MAX_STR_BUFFER_SIZE: usize = 32;

// The maximum supported precision
pub(crate) const MAX_PRECISION: u32 = 28;
#[cfg(not(feature = "legacy-ops"))]
const MAX_PRECISION_I32: i32 = 28;
// 79,228,162,514,264,337,593,543,950,335
const MAX_I128_REPR: i128 = 0x0000_0000_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;

const MIN: Decimal = Decimal {
    flags: 2_147_483_648,
    lo: 4_294_967_295,
    mid: 4_294_967_295,
    hi: 4_294_967_295,
};

const MAX: Decimal = Decimal {
    flags: 0,
    lo: 4_294_967_295,
    mid: 4_294_967_295,
    hi: 4_294_967_295,
};

// Fast access for 10^n where n is 0-9
const POWERS_10: [u32; 10] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
];
// Fast access for 10^n where n is 10-19
#[allow(dead_code)]
const BIG_POWERS_10: [u64; 10] = [
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
    10_000_000_000_000_000_000,
];

/// `UnpackedDecimal` contains unpacked representation of `Decimal` where each component
/// of decimal-format stored in it's own field
#[derive(Clone, Copy, Debug)]
pub struct UnpackedDecimal {
    pub is_negative: bool,
    pub scale: u32,
    pub hi: u32,
    pub mid: u32,
    pub lo: u32,
}

/// `Decimal` represents a 128 bit representation of a fixed-precision decimal number.
/// The finite set of values of type `Decimal` are of the form m / 10<sup>e</sup>,
/// where m is an integer such that -2<sup>96</sup> < m < 2<sup>96</sup>, and e is an integer
/// between 0 and 28 inclusive.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, AsExpression), sql_type = "Numeric")]
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

/// `RoundingStrategy` represents the different strategies that can be used by
/// `round_dp_with_strategy`.
///
/// `RoundingStrategy::BankersRounding` - Rounds toward the nearest even number, e.g. 6.5 -> 6, 7.5 -> 8
/// `RoundingStrategy::RoundHalfUp` - Rounds up if the value >= 5, otherwise rounds down, e.g. 6.5 -> 7,
/// `RoundingStrategy::RoundHalfDown` - Rounds down if the value =< 5, otherwise rounds up, e.g.
/// 6.5 -> 6, 6.51 -> 7
/// 1.4999999 -> 1
/// `RoundingStrategy::RoundDown` - Always round down.
/// `RoundingStrategy::RoundUp` - Always round up.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoundingStrategy {
    BankersRounding,
    RoundHalfUp,
    RoundHalfDown,
    RoundDown,
    RoundUp,
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
    ///
    /// let pi = Decimal::new(3141, 3);
    /// assert_eq!(pi.to_string(), "3.141");
    /// ```
    pub fn new(num: i64, scale: u32) -> Decimal {
        if scale > MAX_PRECISION {
            panic!(
                "Scale exceeds the maximum precision allowed: {} > {}",
                scale, MAX_PRECISION
            );
        }
        let flags: u32 = scale << SCALE_SHIFT;
        if num < 0 {
            let pos_num = num.wrapping_neg() as u64;
            return Decimal {
                flags: flags | SIGN_MASK,
                hi: 0,
                lo: (pos_num & U32_MASK) as u32,
                mid: ((pos_num >> 32) & U32_MASK) as u32,
            };
        }
        Decimal {
            flags,
            hi: 0,
            lo: (num as u64 & U32_MASK) as u32,
            mid: ((num as u64 >> 32) & U32_MASK) as u32,
        }
    }

    /// Creates a `Decimal` using a 128 bit signed `m` representation and corresponding `e` scale.
    ///
    /// # Arguments
    ///
    /// * `num` - An i128 that represents the `m` portion of the decimal number
    /// * `scale` - A u32 representing the `e` portion of the decimal number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let pi = Decimal::from_i128_with_scale(3141i128, 3);
    /// assert_eq!(pi.to_string(), "3.141");
    /// ```
    pub fn from_i128_with_scale(num: i128, scale: u32) -> Decimal {
        if scale > MAX_PRECISION {
            panic!(
                "Scale exceeds the maximum precision allowed: {} > {}",
                scale, MAX_PRECISION
            );
        }
        let mut neg = false;
        let mut wrapped = num;
        if num > MAX_I128_REPR {
            panic!("Number exceeds maximum value that can be represented");
        } else if num < -MAX_I128_REPR {
            panic!("Number less than minimum value that can be represented");
        } else if num < 0 {
            neg = true;
            wrapped = -num;
        }
        let flags: u32 = flags(neg, scale);
        Decimal {
            flags,
            lo: (wrapped as u64 & U32_MASK) as u32,
            mid: ((wrapped as u64 >> 32) & U32_MASK) as u32,
            hi: ((wrapped as u128 >> 64) as u64 & U32_MASK) as u32,
        }
    }

    /// Returns a `Decimal` using the instances constituent parts.
    ///
    /// # Arguments
    ///
    /// * `lo` - The low 32 bits of a 96-bit integer.
    /// * `mid` - The middle 32 bits of a 96-bit integer.
    /// * `hi` - The high 32 bits of a 96-bit integer.
    /// * `negative` - `true` to indicate a negative number.
    /// * `scale` - A power of 10 ranging from 0 to 28.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let pi = Decimal::from_parts(1102470952, 185874565, 1703060790, false, 28);
    /// assert_eq!(pi.to_string(), "3.1415926535897932384626433832");
    /// ```
    pub const fn from_parts(lo: u32, mid: u32, hi: u32, negative: bool, scale: u32) -> Decimal {
        Decimal {
            lo,
            mid,
            hi,
            flags: flags(negative, scale),
        }
    }

    pub(crate) const fn from_parts_raw(lo: u32, mid: u32, hi: u32, flags: u32) -> Decimal {
        Decimal { lo, mid, hi, flags }
    }

    /// Returns a `Result` which if successful contains the `Decimal` constitution of
    /// the scientific notation provided by `value`.
    ///
    /// # Arguments
    ///
    /// * `value` - The scientific notation of the `Decimal`.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let value = Decimal::from_scientific("9.7e-7").unwrap();
    /// assert_eq!(value.to_string(), "0.00000097");
    /// ```
    pub fn from_scientific(value: &str) -> Result<Decimal, Error> {
        let err = Error::new("Failed to parse");
        let mut split = value.splitn(2, |c| c == 'e' || c == 'E');

        let base = split.next().ok_or_else(|| err.clone())?;
        let exp = split.next().ok_or_else(|| err.clone())?;

        let mut ret = Decimal::from_str(base)?;
        let current_scale = ret.scale();

        if exp.starts_with('-') {
            let exp: u32 = exp[1..].parse().map_err(move |_| err)?;
            ret.set_scale(current_scale + exp)?;
        } else {
            let exp: u32 = exp.parse().map_err(move |_| err)?;
            if exp <= current_scale {
                ret.set_scale(current_scale - exp)?;
            } else {
                ret *= Decimal::from_i64(10_i64.pow(exp)).unwrap();
                ret = ret.normalize();
            }
        }
        Ok(ret)
    }

    /// Returns the scale of the decimal number, otherwise known as `e`.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let num = Decimal::new(1234, 3);
    /// assert_eq!(num.scale(), 3u32);
    /// ```
    #[inline]
    pub const fn scale(&self) -> u32 {
        ((self.flags & SCALE_MASK) >> SCALE_SHIFT) as u32
    }

    /// An optimized method for changing the sign of a decimal number.
    ///
    /// # Arguments
    ///
    /// * `positive`: true if the resulting decimal should be positive.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let mut one = Decimal::new(1, 0);
    /// one.set_sign(false);
    /// assert_eq!(one.to_string(), "-1");
    /// ```
    #[deprecated(since = "1.4.0", note = "please use `set_sign_positive` instead")]
    pub fn set_sign(&mut self, positive: bool) {
        self.set_sign_positive(positive);
    }

    /// An optimized method for changing the sign of a decimal number.
    ///
    /// # Arguments
    ///
    /// * `positive`: true if the resulting decimal should be positive.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let mut one = Decimal::new(1, 0);
    /// one.set_sign_positive(false);
    /// assert_eq!(one.to_string(), "-1");
    /// ```
    #[inline(always)]
    pub fn set_sign_positive(&mut self, positive: bool) {
        if positive {
            self.flags &= UNSIGN_MASK;
        } else {
            self.flags |= SIGN_MASK;
        }
    }

    /// An optimized method for changing the sign of a decimal number.
    ///
    /// # Arguments
    ///
    /// * `negative`: true if the resulting decimal should be negative.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let mut one = Decimal::new(1, 0);
    /// one.set_sign_negative(true);
    /// assert_eq!(one.to_string(), "-1");
    /// ```
    #[inline(always)]
    pub fn set_sign_negative(&mut self, negative: bool) {
        self.set_sign_positive(!negative);
    }

    /// An optimized method for changing the scale of a decimal number.
    ///
    /// # Arguments
    ///
    /// * `scale`: the new scale of the number
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let mut one = Decimal::new(1, 0);
    /// one.set_scale(5);
    /// assert_eq!(one.to_string(), "0.00001");
    /// ```
    pub fn set_scale(&mut self, scale: u32) -> Result<(), Error> {
        if scale > MAX_PRECISION {
            return Err(Error::new("Scale exceeds maximum precision"));
        }
        self.flags = (scale << SCALE_SHIFT) | (self.flags & SIGN_MASK);
        Ok(())
    }

    /// Modifies the `Decimal` to the given scale, attempting to do so without changing the
    /// underlying number itself.
    ///
    /// Note that setting the scale to something less then the current `Decimal`s scale will
    /// cause the newly created `Decimal` to have some rounding.
    /// Scales greater than the maximum precision supported by `Decimal` will be automatically
    /// rounded to `Decimal::MAX_PRECISION`.
    /// Rounding leverages the half up strategy.
    ///
    /// # Arguments
    /// * `scale`: The scale to use for the new `Decimal` number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let mut number = Decimal::new(1_123, 3);
    /// number.rescale(6);
    /// assert_eq!(number, Decimal::new(1_123_000, 6));
    /// let mut round = Decimal::new(145, 2);
    /// round.rescale(1);
    /// assert_eq!(round, Decimal::new(15, 1));
    /// ```
    pub fn rescale(&mut self, scale: u32) {
        let mut array = [self.lo, self.mid, self.hi];
        let mut value_scale = self.scale();
        rescale_internal(&mut array, &mut value_scale, scale);
        self.lo = array[0];
        self.mid = array[1];
        self.hi = array[2];
        self.flags = flags(self.is_sign_negative(), value_scale);
    }

    /// Returns a serialized version of the decimal number.
    /// The resulting byte array will have the following representation:
    ///
    /// * Bytes 1-4: flags
    /// * Bytes 5-8: lo portion of `m`
    /// * Bytes 9-12: mid portion of `m`
    /// * Bytes 13-16: high portion of `m`
    pub const fn serialize(&self) -> [u8; 16] {
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
    pub const fn deserialize(bytes: [u8; 16]) -> Decimal {
        Decimal {
            flags: (bytes[0] as u32) | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16 | (bytes[3] as u32) << 24,
            lo: (bytes[4] as u32) | (bytes[5] as u32) << 8 | (bytes[6] as u32) << 16 | (bytes[7] as u32) << 24,
            mid: (bytes[8] as u32) | (bytes[9] as u32) << 8 | (bytes[10] as u32) << 16 | (bytes[11] as u32) << 24,
            hi: (bytes[12] as u32) | (bytes[13] as u32) << 8 | (bytes[14] as u32) << 16 | (bytes[15] as u32) << 24,
        }
    }

    /// Returns `true` if the decimal is negative.
    #[deprecated(since = "0.6.3", note = "please use `is_sign_negative` instead")]
    pub fn is_negative(&self) -> bool {
        self.is_sign_negative()
    }

    /// Returns `true` if the decimal is positive.
    #[deprecated(since = "0.6.3", note = "please use `is_sign_positive` instead")]
    pub fn is_positive(&self) -> bool {
        self.is_sign_positive()
    }

    /// Returns `true` if the sign bit of the decimal is negative.
    #[inline(always)]
    pub const fn is_sign_negative(&self) -> bool {
        self.flags & SIGN_MASK > 0
    }

    /// Returns `true` if the sign bit of the decimal is positive.
    #[inline(always)]
    pub const fn is_sign_positive(&self) -> bool {
        self.flags & SIGN_MASK == 0
    }

    /// Returns the minimum possible number that `Decimal` can represent.
    pub const fn min_value() -> Decimal {
        MIN
    }

    /// Returns the maximum possible number that `Decimal` can represent.
    pub const fn max_value() -> Decimal {
        MAX
    }

    /// Returns a new `Decimal` integral with no fractional portion.
    /// This is a true truncation whereby no rounding is performed.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let pi = Decimal::new(3141, 3);
    /// let trunc = Decimal::new(3, 0);
    /// // note that it returns a decimal
    /// assert_eq!(pi.trunc(), trunc);
    /// ```
    pub fn trunc(&self) -> Decimal {
        let mut scale = self.scale();
        if scale == 0 {
            // Nothing to do
            return *self;
        }
        let mut working = [self.lo, self.mid, self.hi];
        while scale > 0 {
            // We're removing precision, so we don't care about overflow
            if scale < 10 {
                div_by_u32(&mut working, POWERS_10[scale as usize]);
                break;
            } else {
                div_by_u32(&mut working, POWERS_10[9]);
                // Only 9 as this array starts with 1
                scale -= 9;
            }
        }
        Decimal {
            lo: working[0],
            mid: working[1],
            hi: working[2],
            flags: flags(self.is_sign_negative(), 0),
        }
    }

    /// Returns a new `Decimal` representing the fractional portion of the number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let pi = Decimal::new(3141, 3);
    /// let fract = Decimal::new(141, 3);
    /// // note that it returns a decimal
    /// assert_eq!(pi.fract(), fract);
    /// ```
    pub fn fract(&self) -> Decimal {
        // This is essentially the original number minus the integral.
        // Could possibly be optimized in the future
        *self - self.trunc()
    }

    /// Computes the absolute value of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let num = Decimal::new(-3141, 3);
    /// assert_eq!(num.abs().to_string(), "3.141");
    /// ```
    pub fn abs(&self) -> Decimal {
        let mut me = *self;
        me.set_sign_positive(true);
        me
    }

    /// Returns the largest integer less than or equal to a number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let num = Decimal::new(3641, 3);
    /// assert_eq!(num.floor().to_string(), "3");
    /// ```
    pub fn floor(&self) -> Decimal {
        let scale = self.scale();
        if scale == 0 {
            // Nothing to do
            return *self;
        }

        // Opportunity for optimization here
        let floored = self.trunc();
        if self.is_sign_negative() && !self.fract().is_zero() {
            floored - Decimal::one()
        } else {
            floored
        }
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let num = Decimal::new(3141, 3);
    /// assert_eq!(num.ceil().to_string(), "4");
    /// let num = Decimal::new(3, 0);
    /// assert_eq!(num.ceil().to_string(), "3");
    /// ```
    pub fn ceil(&self) -> Decimal {
        let scale = self.scale();
        if scale == 0 {
            // Nothing to do
            return *self;
        }

        // Opportunity for optimization here
        if self.is_sign_positive() && !self.fract().is_zero() {
            self.trunc() + Decimal::one()
        } else {
            self.trunc()
        }
    }

    /// Returns the maximum of the two numbers.
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let x = Decimal::new(1, 0);
    /// let y = Decimal::new(2, 0);
    /// assert_eq!(y, x.max(y));
    /// ```
    pub fn max(self, other: Decimal) -> Decimal {
        if self < other {
            other
        } else {
            self
        }
    }

    /// Returns the minimum of the two numbers.
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let x = Decimal::new(1, 0);
    /// let y = Decimal::new(2, 0);
    /// assert_eq!(x, x.min(y));
    /// ```
    pub fn min(self, other: Decimal) -> Decimal {
        if self > other {
            other
        } else {
            self
        }
    }

    /// Strips any trailing zero's from a `Decimal` and converts -0 to 0.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// let number = Decimal::new(3100, 3);
    /// // note that it returns a decimal, without the extra scale
    /// assert_eq!(number.normalize().to_string(), "3.1");
    /// ```
    pub fn normalize(&self) -> Decimal {
        if self.is_zero() {
            // Convert -0, -0.0*, or 0.0* to 0.
            return Decimal::zero();
        }

        let mut scale = self.scale();
        if scale == 0 {
            // Nothing to do
            return *self;
        }

        let mut result = [self.lo, self.mid, self.hi];
        let mut working = [self.lo, self.mid, self.hi];
        while scale > 0 {
            if div_by_u32(&mut working, 10) > 0 {
                break;
            }
            scale -= 1;
            result.copy_from_slice(&working);
        }
        Decimal {
            lo: result[0],
            mid: result[1],
            hi: result[2],
            flags: flags(self.is_sign_negative(), scale),
        }
    }

    /// Returns a new `Decimal` number with no fractional portion (i.e. an integer).
    /// Rounding currently follows "Bankers Rounding" rules. e.g. 6.5 -> 6, 7.5 -> 8
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    ///
    /// // Demonstrating bankers rounding...
    /// let number_down = Decimal::new(65, 1);
    /// let number_up   = Decimal::new(75, 1);
    /// assert_eq!(number_down.round().to_string(), "6");
    /// assert_eq!(number_up.round().to_string(), "8");
    /// ```
    pub fn round(&self) -> Decimal {
        self.round_dp(0)
    }

    /// Returns a new `Decimal` number with the specified number of decimal points for fractional
    /// portion.
    /// Rounding is performed using the provided [`RoundingStrategy`]
    ///
    /// # Arguments
    /// * `dp`: the number of decimal points to round to.
    /// * `strategy`: the [`RoundingStrategy`] to use.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::{Decimal, RoundingStrategy};
    /// use core::str::FromStr;
    ///
    /// let tax = Decimal::from_str("3.4395").unwrap();
    /// assert_eq!(tax.round_dp_with_strategy(2, RoundingStrategy::RoundHalfUp).to_string(), "3.44");
    /// ```
    pub fn round_dp_with_strategy(&self, dp: u32, strategy: RoundingStrategy) -> Decimal {
        // Short circuit for zero
        if self.is_zero() {
            return Decimal {
                lo: 0,
                mid: 0,
                hi: 0,
                flags: flags(self.is_sign_negative(), dp),
            };
        }

        let old_scale = self.scale();

        // return early if decimal has a smaller number of fractional places than dp
        // e.g. 2.51 rounded to 3 decimal places is 2.51
        if old_scale <= dp {
            return *self;
        }

        let mut value = [self.lo, self.mid, self.hi];
        let mut value_scale = self.scale();
        let negative = self.is_sign_negative();

        value_scale -= dp;

        // Rescale to zero so it's easier to work with
        while value_scale > 0 {
            if value_scale < 10 {
                div_by_u32(&mut value, POWERS_10[value_scale as usize]);
                value_scale = 0;
            } else {
                div_by_u32(&mut value, POWERS_10[9]);
                value_scale -= 9;
            }
        }

        // Do some midpoint rounding checks
        // We're actually doing two things here.
        //  1. Figuring out midpoint rounding when we're right on the boundary. e.g. 2.50000
        //  2. Figuring out whether to add one or not e.g. 2.51
        // For this, we need to figure out the fractional portion that is additional to
        // the rounded number. e.g. for 0.12345 rounding to 2dp we'd want 345.
        // We're doing the equivalent of losing precision (e.g. to get 0.12)
        // then increasing the precision back up to 0.12000
        let mut offset = [self.lo, self.mid, self.hi];
        let mut diff = old_scale - dp;

        while diff > 0 {
            if diff < 10 {
                div_by_u32(&mut offset, POWERS_10[diff as usize]);
                break;
            } else {
                div_by_u32(&mut offset, POWERS_10[9]);
                // Only 9 as this array starts with 1
                diff -= 9;
            }
        }

        let mut diff = old_scale - dp;

        while diff > 0 {
            if diff < 10 {
                mul_by_u32(&mut offset, POWERS_10[diff as usize]);
                break;
            } else {
                mul_by_u32(&mut offset, POWERS_10[9]);
                // Only 9 as this array starts with 1
                diff -= 9;
            }
        }

        let mut decimal_portion = [self.lo, self.mid, self.hi];
        sub_by_internal(&mut decimal_portion, &offset);

        // If the decimal_portion is zero then we round based on the other data
        let mut cap = [5, 0, 0];
        for _ in 0..(old_scale - dp - 1) {
            mul_by_u32(&mut cap, 10);
        }
        let order = cmp_internal(&decimal_portion, &cap);

        match strategy {
            RoundingStrategy::BankersRounding => {
                match order {
                    Ordering::Equal => {
                        if (value[0] & 1) == 1 {
                            add_one_internal(&mut value);
                        }
                    }
                    Ordering::Greater => {
                        // Doesn't matter about the decimal portion
                        add_one_internal(&mut value);
                    }
                    _ => {}
                }
            }
            RoundingStrategy::RoundHalfDown => {
                if let Ordering::Greater = order {
                    add_one_internal(&mut value);
                }
            }
            RoundingStrategy::RoundHalfUp => {
                // when Ordering::Equal, decimal_portion is 0.5 exactly
                // when Ordering::Greater, decimal_portion is > 0.5
                match order {
                    Ordering::Equal => {
                        add_one_internal(&mut value);
                    }
                    Ordering::Greater => {
                        // Doesn't matter about the decimal portion
                        add_one_internal(&mut value);
                    }
                    _ => {}
                }
            }
            RoundingStrategy::RoundUp => {
                if !is_all_zero(&decimal_portion) {
                    add_one_internal(&mut value);
                }
            }
            RoundingStrategy::RoundDown => (),
        }

        Decimal {
            lo: value[0],
            mid: value[1],
            hi: value[2],
            flags: flags(negative, dp),
        }
    }

    /// Returns a new `Decimal` number with the specified number of decimal points for fractional portion.
    /// Rounding currently follows "Bankers Rounding" rules. e.g. 6.5 -> 6, 7.5 -> 8
    ///
    /// # Arguments
    /// * `dp`: the number of decimal points to round to.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    /// use core::str::FromStr;
    ///
    /// let pi = Decimal::from_str("3.1415926535897932384626433832").unwrap();
    /// assert_eq!(pi.round_dp(2).to_string(), "3.14");
    /// ```
    pub fn round_dp(&self, dp: u32) -> Decimal {
        self.round_dp_with_strategy(dp, RoundingStrategy::BankersRounding)
    }

    /// Convert `Decimal` to an internal representation of the underlying struct. This is useful
    /// for debugging the internal state of the object.
    ///
    /// # Important Disclaimer
    /// This is primarily intended for library maintainers. The internal representation of a
    /// `Decimal` is considered "unstable" for public use.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    /// use core::str::FromStr;
    ///
    /// let pi = Decimal::from_str("3.1415926535897932384626433832").unwrap();
    /// assert_eq!(format!("{:?}", pi), "3.1415926535897932384626433832");
    /// assert_eq!(format!("{:?}", pi.unpack()), "UnpackedDecimal { \
    ///     is_negative: false, scale: 28, hi: 1703060790, mid: 185874565, lo: 1102470952 \
    /// }");
    /// ```
    pub const fn unpack(&self) -> UnpackedDecimal {
        UnpackedDecimal {
            is_negative: self.is_sign_negative(),
            scale: self.scale(),
            hi: self.hi,
            lo: self.lo,
            mid: self.mid,
        }
    }

    /// Convert `Decimal` to an internal representation of the underlying struct. This is useful
    /// for debugging the internal state of the object.
    ///
    /// # Important Disclaimer
    /// This is primarily intended for library maintainers. The internal representation of a
    /// `Decimal` is considered "unstable" for public use.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal::Decimal;
    /// use core::str::FromStr;
    ///
    /// let pi = Decimal::from_str("3.1415926535897932384626433832").unwrap();
    /// assert_eq!(format!("{:?}", pi), "3.1415926535897932384626433832");
    /// assert_eq!(format!("{:?}", pi.unpack()), "UnpackedDecimal { \
    ///     is_negative: false, scale: 28, hi: 1703060790, mid: 185874565, lo: 1102470952 \
    /// }");
    /// ```

    #[inline(always)]
    pub(crate) const fn mantissa_array3(&self) -> [u32; 3] {
        [self.lo, self.mid, self.hi]
    }

    #[inline(always)]
    pub(crate) const fn mantissa_array4(&self) -> [u32; 4] {
        [self.lo, self.mid, self.hi, 0]
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
                shl1_internal(bits, 0);
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
                add_one_internal(bits);
            }
        }

        // This step is required in order to remove excess bits of precision from the
        // end of the bit representation, down to the precision guaranteed by the
        // floating point number
        if is64 {
            // Guaranteed to about 16 dp
            while exponent10 < 0 && (bits[2] != 0 || (bits[1] & 0xFFF0_0000) != 0) {
                let rem10 = div_by_u32(bits, 10);
                exponent10 += 1;
                if rem10 >= 5 {
                    add_one_internal(bits);
                }
            }
        } else {
            // Guaranteed to about 7 dp
            while exponent10 < 0
                && (bits[2] != 0 || bits[1] != 0 || (bits[2] == 0 && bits[1] == 0 && (bits[0] & 0xFF00_0000) != 0))
            {
                let rem10 = div_by_u32(bits, 10);
                exponent10 += 1;
                if rem10 >= 5 {
                    add_one_internal(bits);
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

        Some(Decimal {
            lo: bits[0],
            mid: bits[1],
            hi: bits[2],
            flags: flags(!positive, -exponent10 as u32),
        })
    }

    /// Checked addition. Computes `self + other`, returning `None` if overflow occurred.
    #[inline(always)]
    pub fn checked_add(self, other: Decimal) -> Option<Decimal> {
        match ops::add_impl(&self, &other) {
            CalculationResult::Ok(result) => Some(result),
            CalculationResult::Overflow => None,
            _ => None,
        }
    }

    /// Checked subtraction. Computes `self - other`, returning `None` if overflow occurred.
    #[inline(always)]
    pub fn checked_sub(self, other: Decimal) -> Option<Decimal> {
        match ops::sub_impl(&self, &other) {
            CalculationResult::Ok(result) => Some(result),
            CalculationResult::Overflow => None,
            _ => None,
        }
    }

    /// Checked multiplication. Computes `self * other`, returning `None` if overflow occurred.
    #[inline]
    pub fn checked_mul(self, other: Decimal) -> Option<Decimal> {
        match ops::mul_impl(&self, &other) {
            CalculationResult::Ok(result) => Some(result),
            CalculationResult::Overflow => None,
            _ => None,
        }
    }

    /// Checked division. Computes `self / other`, returning `None` if `other == 0.0` or the
    /// division results in overflow.
    pub fn checked_div(self, other: Decimal) -> Option<Decimal> {
        match ops::div_impl(&self, &other) {
            CalculationResult::Ok(quot) => Some(quot),
            CalculationResult::Overflow => None,
            CalculationResult::DivByZero => None,
        }
    }

    /// Checked remainder. Computes `self % other`, returning `None` if `other == 0.0`.
    pub fn checked_rem(self, other: Decimal) -> Option<Decimal> {
        match ops::rem_impl(&self, &other) {
            CalculationResult::Ok(quot) => Some(quot),
            CalculationResult::Overflow => None,
            CalculationResult::DivByZero => None,
        }
    }

    pub fn from_str_radix(str: &str, radix: u32) -> Result<Self, crate::Error> {
        if radix == 10 {
            parse_str_radix_10(str)
        } else {
            parse_str_radix_n(str, radix)
        }
    }
}

impl Default for Decimal {
    fn default() -> Self {
        Self::zero()
    }
}

pub(crate) enum CalculationResult {
    Ok(Decimal),
    Overflow,
    DivByZero,
}

#[inline]
const fn flags(neg: bool, scale: u32) -> u32 {
    (scale << SCALE_SHIFT) | ((neg as u32) << SIGN_SHIFT)
}

/// Rescales the given decimals to equivalent scales.
/// It will firstly try to scale both the left and the right side to
/// the maximum scale of left/right. If it is unable to do that it
/// will try to reduce the accuracy of the other argument.
/// e.g. with 1.23 and 2.345 it'll rescale the first arg to 1.230
#[inline(always)]
fn rescale_to_maximum_scale(left: &mut [u32; 3], left_scale: &mut u32, right: &mut [u32; 3], right_scale: &mut u32) {
    if left_scale == right_scale {
        // Nothing to do
        return;
    }

    if is_all_zero(left) {
        *left_scale = *right_scale;
        return;
    } else if is_all_zero(right) {
        *right_scale = *left_scale;
        return;
    }

    if left_scale > right_scale {
        rescale_internal(right, right_scale, *left_scale);
        if right_scale != left_scale {
            rescale_internal(left, left_scale, *right_scale);
        }
    } else {
        rescale_internal(left, left_scale, *right_scale);
        if right_scale != left_scale {
            rescale_internal(right, right_scale, *left_scale);
        }
    }
}

/// Rescales the given decimal to new scale.
/// e.g. with 1.23 and new scale 3 rescale the value to 1.230
#[inline(always)]
fn rescale_internal(value: &mut [u32; 3], value_scale: &mut u32, new_scale: u32) {
    if *value_scale == new_scale {
        // Nothing to do
        return;
    }

    if is_all_zero(value) {
        *value_scale = new_scale;
        return;
    }

    if *value_scale > new_scale {
        let mut diff = *value_scale - new_scale;
        // Scaling further isn't possible since we got an overflow
        // In this case we need to reduce the accuracy of the "side to keep"

        // Now do the necessary rounding
        let mut remainder = 0;
        while diff > 0 {
            if is_all_zero(value) {
                *value_scale = new_scale;
                return;
            }

            diff -= 1;

            // Any remainder is discarded if diff > 0 still (i.e. lost precision)
            remainder = div_by_10(value);
        }
        if remainder >= 5 {
            for part in value.iter_mut() {
                let digit = u64::from(*part) + 1u64;
                remainder = if digit > 0xFFFF_FFFF { 1 } else { 0 };
                *part = (digit & 0xFFFF_FFFF) as u32;
                if remainder == 0 {
                    break;
                }
            }
        }
        *value_scale = new_scale;
    } else {
        let mut diff = new_scale - *value_scale;
        let mut working = [value[0], value[1], value[2]];
        while diff > 0 && mul_by_10(&mut working) == 0 {
            value.copy_from_slice(&working);
            diff -= 1;
        }
        *value_scale = new_scale - diff;
    }
}

fn add_by_internal(value: &mut [u32], by: &[u32]) -> u32 {
    let mut carry: u64 = 0;
    let vl = value.len();
    let bl = by.len();
    if vl >= bl {
        let mut sum: u64;
        for i in 0..bl {
            sum = u64::from(value[i]) + u64::from(by[i]) + carry;
            value[i] = (sum & U32_MASK) as u32;
            carry = sum >> 32;
        }
        if vl > bl && carry > 0 {
            for i in value.iter_mut().skip(bl) {
                sum = u64::from(*i) + carry;
                *i = (sum & U32_MASK) as u32;
                carry = sum >> 32;
                if carry == 0 {
                    break;
                }
            }
        }
    } else if vl + 1 == bl {
        // Overflow, by default, is anything in the high portion of by
        let mut sum: u64;
        for i in 0..vl {
            sum = u64::from(value[i]) + u64::from(by[i]) + carry;
            value[i] = (sum & U32_MASK) as u32;
            carry = sum >> 32;
        }
        if by[vl] > 0 {
            carry += u64::from(by[vl]);
        }
    } else {
        panic!("Internal error: add using incompatible length arrays. {} <- {}", vl, bl);
    }
    carry as u32
}

#[inline]
fn add_one_internal(value: &mut [u32]) -> u32 {
    let mut carry: u64 = 1; // Start with one, since adding one
    let mut sum: u64;
    for i in value.iter_mut() {
        sum = (*i as u64) + carry;
        *i = (sum & U32_MASK) as u32;
        carry = sum >> 32;
    }

    carry as u32
}

fn sub_by_internal(value: &mut [u32], by: &[u32]) -> u32 {
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
    let part = 0x1_0000_0000u64 + u64::from(left) - (u64::from(right) + u64::from(overflow));
    let lo = part as u32;
    let hi = 1 - ((part >> 32) as u32);
    (lo, hi)
}

// Returns overflow
#[inline]
fn mul_by_10(bits: &mut [u32; 3]) -> u32 {
    let mut overflow = 0u64;
    for b in bits.iter_mut() {
        let result = u64::from(*b) * 10u64 + overflow;
        let hi = (result >> 32) & U32_MASK;
        let lo = (result & U32_MASK) as u32;
        *b = lo;
        overflow = hi;
    }

    overflow as u32
}

// Returns overflow
pub(crate) fn mul_by_u32(bits: &mut [u32], m: u32) -> u32 {
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
    let hi = ((result >> 32) & U32_MASK) as u32;
    let lo = (result & U32_MASK) as u32;
    (lo, hi)
}

#[cfg(feature = "legacy-ops")]
mod ops {
    use super::*;

    pub(crate) fn add_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        // Convert to the same scale
        let mut my = [d1.lo, d1.mid, d1.hi];
        let mut my_scale = d1.scale();
        let mut ot = [d2.lo, d2.mid, d2.hi];
        let mut other_scale = d2.scale();
        rescale_to_maximum_scale(&mut my, &mut my_scale, &mut ot, &mut other_scale);
        let mut final_scale = my_scale.max(other_scale);

        // Add the items together
        let my_negative = d1.is_sign_negative();
        let other_negative = d2.is_sign_negative();
        let mut negative = false;
        let carry;
        if !(my_negative ^ other_negative) {
            negative = my_negative;
            carry = add_by_internal3(&mut my, &ot);
        } else {
            let cmp = cmp_internal(&my, &ot);
            // -x + y
            // if x > y then it's negative (i.e. -2 + 1)
            match cmp {
                Ordering::Less => {
                    negative = other_negative;
                    sub_by_internal3(&mut ot, &my);
                    my[0] = ot[0];
                    my[1] = ot[1];
                    my[2] = ot[2];
                }
                Ordering::Greater => {
                    negative = my_negative;
                    sub_by_internal3(&mut my, &ot);
                }
                Ordering::Equal => {
                    // -2 + 2
                    my[0] = 0;
                    my[1] = 0;
                    my[2] = 0;
                }
            }
            carry = 0;
        }

        // If we have a carry we underflowed.
        // We need to lose some significant digits (if possible)
        if carry > 0 {
            if final_scale == 0 {
                return CalculationResult::Overflow;
            }

            // Copy it over to a temp array for modification
            let mut temp = [my[0], my[1], my[2], carry];
            while final_scale > 0 && temp[3] != 0 {
                div_by_u32(&mut temp, 10);
                final_scale -= 1;
            }

            // If we still have a carry bit then we overflowed
            if temp[3] > 0 {
                return CalculationResult::Overflow;
            }

            // Copy it back - we're done
            my[0] = temp[0];
            my[1] = temp[1];
            my[2] = temp[2];
        }

        CalculationResult::Ok(Decimal {
            lo: my[0],
            mid: my[1],
            hi: my[2],
            flags: flags(negative, final_scale),
        })
    }

    pub(crate) fn sub_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        let negated_d2 = Decimal {
            lo: d2.lo,
            mid: d2.mid,
            hi: d2.hi,
            flags: d2.flags ^ SIGN_MASK,
        };
        add_impl(d1, &negated_d2)
    }

    pub(crate) fn div_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        if d2.is_zero() {
            return CalculationResult::DivByZero;
        }
        if d1.is_zero() {
            return CalculationResult::Ok(Decimal::zero());
        }

        let dividend = [d1.lo, d1.mid, d1.hi];
        let divisor = [d2.lo, d2.mid, d2.hi];
        let mut quotient = [0u32, 0u32, 0u32];
        let mut quotient_scale: i32 = d1.scale() as i32 - d2.scale() as i32;

        // We supply an extra overflow word for each of the dividend and the remainder
        let mut working_quotient = [dividend[0], dividend[1], dividend[2], 0u32];
        let mut working_remainder = [0u32, 0u32, 0u32, 0u32];
        let mut working_scale = quotient_scale;
        let mut remainder_scale = quotient_scale;
        let mut underflow;

        loop {
            div_internal(&mut working_quotient, &mut working_remainder, &divisor);
            underflow = add_with_scale_internal(
                &mut quotient,
                &mut quotient_scale,
                &mut working_quotient,
                &mut working_scale,
            );

            // Multiply the remainder by 10
            let mut overflow = 0;
            for part in working_remainder.iter_mut() {
                let (lo, hi) = mul_part(*part, 10, overflow);
                *part = lo;
                overflow = hi;
            }
            // Copy temp remainder into the temp quotient section
            working_quotient.copy_from_slice(&working_remainder);

            remainder_scale += 1;
            working_scale = remainder_scale;

            if underflow || is_all_zero(&working_remainder) {
                break;
            }
        }

        // If we have a really big number try to adjust the scale to 0
        while quotient_scale < 0 {
            copy_array_diff_lengths(&mut working_quotient, &quotient);
            working_quotient[3] = 0;
            working_remainder.iter_mut().for_each(|x| *x = 0);

            // Mul 10
            let mut overflow = 0;
            for part in &mut working_quotient {
                let (lo, hi) = mul_part(*part, 10, overflow);
                *part = lo;
                overflow = hi;
            }
            for part in &mut working_remainder {
                let (lo, hi) = mul_part(*part, 10, overflow);
                *part = lo;
                overflow = hi;
            }
            if working_quotient[3] == 0 && is_all_zero(&working_remainder) {
                quotient_scale += 1;
                quotient[0] = working_quotient[0];
                quotient[1] = working_quotient[1];
                quotient[2] = working_quotient[2];
            } else {
                // Overflow
                return CalculationResult::Overflow;
            }
        }

        if quotient_scale > 255 {
            quotient[0] = 0;
            quotient[1] = 0;
            quotient[2] = 0;
            quotient_scale = 0;
        }

        let mut quotient_negative = d1.is_sign_negative() ^ d2.is_sign_negative();

        // Check for underflow
        let mut final_scale: u32 = quotient_scale as u32;
        if final_scale > MAX_PRECISION {
            let mut remainder = 0;

            // Division underflowed. We must remove some significant digits over using
            //  an invalid scale.
            while final_scale > MAX_PRECISION && !is_all_zero(&quotient) {
                remainder = div_by_u32(&mut quotient, 10);
                final_scale -= 1;
            }
            if final_scale > MAX_PRECISION {
                // Result underflowed so set to zero
                final_scale = 0;
                quotient_negative = false;
            } else if remainder >= 5 {
                for part in &mut quotient {
                    if remainder == 0 {
                        break;
                    }
                    let digit: u64 = u64::from(*part) + 1;
                    remainder = if digit > 0xFFFF_FFFF { 1 } else { 0 };
                    *part = (digit & 0xFFFF_FFFF) as u32;
                }
            }
        }

        CalculationResult::Ok(Decimal {
            lo: quotient[0],
            mid: quotient[1],
            hi: quotient[2],
            flags: flags(quotient_negative, final_scale),
        })
    }

    pub(crate) fn mul_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        // Early exit if either is zero
        if d1.is_zero() || d2.is_zero() {
            return CalculationResult::Ok(Decimal::zero());
        }

        // We are only resulting in a negative if we have mismatched signs
        let negative = d1.is_sign_negative() ^ d2.is_sign_negative();

        // We get the scale of the result by adding the operands. This may be too big, however
        //  we'll correct later
        let mut final_scale = d1.scale() + d2.scale();

        // First of all, if ONLY the lo parts of both numbers is filled
        // then we can simply do a standard 64 bit calculation. It's a minor
        // optimization however prevents the need for long form multiplication
        if d1.mid == 0 && d1.hi == 0 && d2.mid == 0 && d2.hi == 0 {
            // Simply multiplication
            let mut u64_result = u64_to_array(u64::from(d1.lo) * u64::from(d2.lo));

            // If we're above max precision then this is a very small number
            if final_scale > MAX_PRECISION {
                final_scale -= MAX_PRECISION;

                // If the number is above 19 then this will equate to zero.
                // This is because the max value in 64 bits is 1.84E19
                if final_scale > 19 {
                    return CalculationResult::Ok(Decimal::zero());
                }

                let mut rem_lo = 0;
                let mut power;
                if final_scale > 9 {
                    // Since 10^10 doesn't fit into u32, we divide by 10^10/4
                    // and multiply the next divisor by 4.
                    rem_lo = div_by_u32(&mut u64_result, 2_500_000_000);
                    power = POWERS_10[final_scale as usize - 10] << 2;
                } else {
                    power = POWERS_10[final_scale as usize];
                }

                // Divide fits in 32 bits
                let rem_hi = div_by_u32(&mut u64_result, power);

                // Round the result. Since the divisor is a power of 10
                // we check to see if the remainder is >= 1/2 divisor
                power >>= 1;
                if rem_hi >= power && (rem_hi > power || (rem_lo | (u64_result[0] & 0x1)) != 0) {
                    u64_result[0] += 1;
                }

                final_scale = MAX_PRECISION;
            }
            return CalculationResult::Ok(Decimal {
                lo: u64_result[0],
                mid: u64_result[1],
                hi: 0,
                flags: flags(negative, final_scale),
            });
        }

        // We're using some of the high bits, so we essentially perform
        // long form multiplication. We compute the 9 partial products
        // into a 192 bit result array.
        //
        //                     [my-h][my-m][my-l]
        //                  x  [ot-h][ot-m][ot-l]
        // --------------------------------------
        // 1.                        [r-hi][r-lo] my-l * ot-l [0, 0]
        // 2.                  [r-hi][r-lo]       my-l * ot-m [0, 1]
        // 3.                  [r-hi][r-lo]       my-m * ot-l [1, 0]
        // 4.            [r-hi][r-lo]             my-m * ot-m [1, 1]
        // 5.            [r-hi][r-lo]             my-l * ot-h [0, 2]
        // 6.            [r-hi][r-lo]             my-h * ot-l [2, 0]
        // 7.      [r-hi][r-lo]                   my-m * ot-h [1, 2]
        // 8.      [r-hi][r-lo]                   my-h * ot-m [2, 1]
        // 9.[r-hi][r-lo]                         my-h * ot-h [2, 2]
        let my = [d1.lo, d1.mid, d1.hi];
        let ot = [d2.lo, d2.mid, d2.hi];
        let mut product = [0u32, 0u32, 0u32, 0u32, 0u32, 0u32];

        // We can perform a minor short circuit here. If the
        // high portions are both 0 then we can skip portions 5-9
        let to = if my[2] == 0 && ot[2] == 0 { 2 } else { 3 };

        for my_index in 0..to {
            for ot_index in 0..to {
                let (mut rlo, mut rhi) = mul_part(my[my_index], ot[ot_index], 0);

                // Get the index for the lo portion of the product
                for prod in product.iter_mut().skip(my_index + ot_index) {
                    let (res, overflow) = add_part(rlo, *prod);
                    *prod = res;

                    // If we have something in rhi from before then promote that
                    if rhi > 0 {
                        // If we overflowed in the last add, add that with rhi
                        if overflow > 0 {
                            let (nlo, nhi) = add_part(rhi, overflow);
                            rlo = nlo;
                            rhi = nhi;
                        } else {
                            rlo = rhi;
                            rhi = 0;
                        }
                    } else if overflow > 0 {
                        rlo = overflow;
                        rhi = 0;
                    } else {
                        break;
                    }

                    // If nothing to do next round then break out
                    if rlo == 0 {
                        break;
                    }
                }
            }
        }

        // If our result has used up the high portion of the product
        // then we either have an overflow or an underflow situation
        // Overflow will occur if we can't scale it back, whereas underflow
        // with kick in rounding
        let mut remainder = 0;
        while final_scale > 0 && (product[3] != 0 || product[4] != 0 || product[5] != 0) {
            remainder = div_by_u32(&mut product, 10u32);
            final_scale -= 1;
        }

        // Round up the carry if we need to
        if remainder >= 5 {
            for part in product.iter_mut() {
                if remainder == 0 {
                    break;
                }
                let digit: u64 = u64::from(*part) + 1;
                remainder = if digit > 0xFFFF_FFFF { 1 } else { 0 };
                *part = (digit & 0xFFFF_FFFF) as u32;
            }
        }

        // If we're still above max precision then we'll try again to
        // reduce precision - we may be dealing with a limit of "0"
        if final_scale > MAX_PRECISION {
            // We're in an underflow situation
            // The easiest way to remove precision is to divide off the result
            while final_scale > MAX_PRECISION && !is_all_zero(&product) {
                div_by_u32(&mut product, 10);
                final_scale -= 1;
            }
            // If we're still at limit then we can't represent any
            // siginificant decimal digits and will return an integer only
            // Can also be invoked while representing 0.
            if final_scale > MAX_PRECISION {
                final_scale = 0;
            }
        } else if !(product[3] == 0 && product[4] == 0 && product[5] == 0) {
            // We're in an overflow situation - we're within our precision bounds
            // but still have bits in overflow
            return CalculationResult::Overflow;
        }

        CalculationResult::Ok(Decimal {
            lo: product[0],
            mid: product[1],
            hi: product[2],
            flags: flags(negative, final_scale),
        })
    }

    pub(crate) fn rem_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        if d2.is_zero() {
            return CalculationResult::DivByZero;
        }
        if d1.is_zero() {
            return CalculationResult::Ok(Decimal::zero());
        }

        // Rescale so comparable
        let initial_scale = d1.scale();
        let mut quotient = [d1.lo, d1.mid, d1.hi];
        let mut quotient_scale = initial_scale;
        let mut divisor = [d2.lo, d2.mid, d2.hi];
        let mut divisor_scale = d2.scale();
        rescale_to_maximum_scale(&mut quotient, &mut quotient_scale, &mut divisor, &mut divisor_scale);

        // Working is the remainder + the quotient
        // We use an aligned array since we'll be using it a lot.
        let mut working_quotient = [quotient[0], quotient[1], quotient[2], 0u32];
        let mut working_remainder = [0u32, 0u32, 0u32, 0u32];
        div_internal(&mut working_quotient, &mut working_remainder, &divisor);

        // Round if necessary. This is for semantic correctness, but could feasibly be removed for
        // performance improvements.
        if quotient_scale > initial_scale {
            let mut working = [
                working_remainder[0],
                working_remainder[1],
                working_remainder[2],
                working_remainder[3],
            ];
            while quotient_scale > initial_scale {
                if div_by_u32(&mut working, 10) > 0 {
                    break;
                }
                quotient_scale -= 1;
                working_remainder.copy_from_slice(&working);
            }
        }

        CalculationResult::Ok(Decimal {
            lo: working_remainder[0],
            mid: working_remainder[1],
            hi: working_remainder[2],
            flags: flags(d1.is_sign_negative(), quotient_scale),
        })
    }

    #[inline]
    fn add_part(left: u32, right: u32) -> (u32, u32) {
        let added = u64::from(left) + u64::from(right);
        ((added & U32_MASK) as u32, (added >> 32 & U32_MASK) as u32)
    }

    #[inline(always)]
    fn sub_by_internal3(value: &mut [u32; 3], by: &[u32; 3]) {
        let mut overflow = 0;
        let vl = value.len();
        for i in 0..vl {
            let part = (0x1_0000_0000u64 + u64::from(value[i])) - (u64::from(by[i]) + overflow);
            value[i] = part as u32;
            overflow = 1 - (part >> 32);
        }
    }

    fn div_internal(quotient: &mut [u32; 4], remainder: &mut [u32; 4], divisor: &[u32; 3]) {
        // There are a couple of ways to do division on binary numbers:
        //   1. Using long division
        //   2. Using the complement method
        // ref: http://paulmason.me/dividing-binary-numbers-part-2/
        // The complement method basically keeps trying to subtract the
        // divisor until it can't anymore and placing the rest in remainder.
        let mut complement = [
            divisor[0] ^ 0xFFFF_FFFF,
            divisor[1] ^ 0xFFFF_FFFF,
            divisor[2] ^ 0xFFFF_FFFF,
            0xFFFF_FFFF,
        ];

        // Add one onto the complement
        add_one_internal4(&mut complement);

        // Make sure the remainder is 0
        remainder.iter_mut().for_each(|x| *x = 0);

        // If we have nothing in our hi+ block then shift over till we do
        let mut blocks_to_process = 0;
        while blocks_to_process < 4 && quotient[3] == 0 {
            // memcpy would be useful here
            quotient[3] = quotient[2];
            quotient[2] = quotient[1];
            quotient[1] = quotient[0];
            quotient[0] = 0;

            // Incremember the counter
            blocks_to_process += 1;
        }

        // Let's try and do the addition...
        let mut block = blocks_to_process << 5;
        let mut working = [0u32, 0u32, 0u32, 0u32];
        while block < 128 {
            // << 1 for quotient AND remainder. Moving the carry from the quotient to the bottom of the
            // remainder.
            let carry = shl1_internal(quotient, 0);
            shl1_internal(remainder, carry);

            // Copy the remainder of working into sub
            working.copy_from_slice(remainder);

            // Add the remainder with the complement
            add_by_internal(&mut working, &complement);

            // Check for the significant bit - move over to the quotient
            // as necessary
            if (working[3] & 0x8000_0000) == 0 {
                remainder.copy_from_slice(&working);
                quotient[0] |= 1;
            }

            // Increment our pointer
            block += 1;
        }
    }

    #[inline]
    fn copy_array_diff_lengths(into: &mut [u32], from: &[u32]) {
        for i in 0..into.len() {
            if i >= from.len() {
                break;
            }
            into[i] = from[i];
        }
    }

    #[inline]
    fn add_one_internal4(value: &mut [u32; 4]) -> u32 {
        let mut carry: u64 = 1; // Start with one, since adding one
        let mut sum: u64;
        for i in value.iter_mut() {
            sum = (*i as u64) + carry;
            *i = (sum & U32_MASK) as u32;
            carry = sum >> 32;
        }

        carry as u32
    }

    #[inline]
    fn add_by_internal3(value: &mut [u32; 3], by: &[u32; 3]) -> u32 {
        let mut carry: u32 = 0;
        let bl = by.len();
        for i in 0..bl {
            let res1 = value[i].overflowing_add(by[i]);
            let res2 = res1.0.overflowing_add(carry);
            value[i] = res2.0;
            carry = (res1.1 | res2.1) as u32;
        }
        carry
    }

    #[inline]
    const fn u64_to_array(value: u64) -> [u32; 2] {
        [(value & U32_MASK) as u32, (value >> 32 & U32_MASK) as u32]
    }

    fn add_with_scale_internal(
        quotient: &mut [u32; 3],
        quotient_scale: &mut i32,
        working_quotient: &mut [u32; 4],
        working_scale: &mut i32,
    ) -> bool {
        // Add quotient and the working (i.e. quotient = quotient + working)
        if is_all_zero(quotient) {
            // Quotient is zero so we can just copy the working quotient in directly
            // First, make sure they are both 96 bit.
            while working_quotient[3] != 0 {
                div_by_u32(working_quotient, 10);
                *working_scale -= 1;
            }
            copy_array_diff_lengths(quotient, working_quotient);
            *quotient_scale = *working_scale;
            return false;
        }

        if is_all_zero(working_quotient) {
            return false;
        }

        // We have ensured that our working is not zero so we should do the addition

        // If our two quotients are different then
        // try to scale down the one with the bigger scale
        let mut temp3 = [0u32, 0u32, 0u32];
        let mut temp4 = [0u32, 0u32, 0u32, 0u32];
        if *quotient_scale != *working_scale {
            // TODO: Remove necessity for temp (without performance impact)
            fn div_by_10(target: &mut [u32], temp: &mut [u32], scale: &mut i32, target_scale: i32) {
                // Copy to the temp array
                temp.copy_from_slice(target);
                // divide by 10 until target scale is reached
                while *scale > target_scale {
                    let remainder = div_by_u32(temp, 10);
                    if remainder == 0 {
                        *scale -= 1;
                        target.copy_from_slice(&temp);
                    } else {
                        break;
                    }
                }
            }

            if *quotient_scale < *working_scale {
                div_by_10(working_quotient, &mut temp4, working_scale, *quotient_scale);
            } else {
                div_by_10(quotient, &mut temp3, quotient_scale, *working_scale);
            }
        }

        // If our two quotients are still different then
        // try to scale up the smaller scale
        if *quotient_scale != *working_scale {
            // TODO: Remove necessity for temp (without performance impact)
            fn mul_by_10(target: &mut [u32], temp: &mut [u32], scale: &mut i32, target_scale: i32) {
                temp.copy_from_slice(target);
                let mut overflow = 0;
                // Multiply by 10 until target scale reached or overflow
                while *scale < target_scale && overflow == 0 {
                    overflow = mul_by_u32(temp, 10);
                    if overflow == 0 {
                        // Still no overflow
                        *scale += 1;
                        target.copy_from_slice(&temp);
                    }
                }
            }

            if *quotient_scale > *working_scale {
                mul_by_10(working_quotient, &mut temp4, working_scale, *quotient_scale);
            } else {
                mul_by_10(quotient, &mut temp3, quotient_scale, *working_scale);
            }
        }

        // If our two quotients are still different then
        // try to scale down the one with the bigger scale
        // (ultimately losing significant digits)
        if *quotient_scale != *working_scale {
            // TODO: Remove necessity for temp (without performance impact)
            fn div_by_10_lossy(target: &mut [u32], temp: &mut [u32], scale: &mut i32, target_scale: i32) {
                temp.copy_from_slice(target);
                // divide by 10 until target scale is reached
                while *scale > target_scale {
                    div_by_u32(temp, 10);
                    *scale -= 1;
                    target.copy_from_slice(&temp);
                }
            }
            if *quotient_scale < *working_scale {
                div_by_10_lossy(working_quotient, &mut temp4, working_scale, *quotient_scale);
            } else {
                div_by_10_lossy(quotient, &mut temp3, quotient_scale, *working_scale);
            }
        }

        // If quotient or working are zero we have an underflow condition
        if is_all_zero(quotient) || is_all_zero(working_quotient) {
            // Underflow
            return true;
        } else {
            // Both numbers have the same scale and can be added.
            // We just need to know whether we can fit them in
            let mut underflow = false;
            let mut temp = [0u32, 0u32, 0u32];
            while !underflow {
                temp.copy_from_slice(quotient);

                // Add the working quotient
                let overflow = add_by_internal(&mut temp, working_quotient);
                if overflow == 0 {
                    // addition was successful
                    quotient.copy_from_slice(&temp);
                    break;
                } else {
                    // addition overflowed - remove significant digits and try again
                    div_by_u32(quotient, 10);
                    *quotient_scale -= 1;
                    div_by_u32(working_quotient, 10);
                    *working_scale -= 1;
                    // Check for underflow
                    underflow = is_all_zero(quotient) || is_all_zero(working_quotient);
                }
            }
            if underflow {
                return true;
            }
        }
        false
    }
}

// This code (in fact, this library) is heavily inspired by the dotnet Decimal number library
// implementation. Consequently, a huge thank you for to all the contributors to that project
// which has also found it's way into here.
#[cfg(not(feature = "legacy-ops"))]
mod ops {
    use super::*;
    use core::ops::BitXor;

    // This is a table of the largest values that will not overflow when multiplied
    // by a given power as represented by the index.
    static POWER_OVERFLOW_VALUES: [Dec12; 8] = [
        Dec12 {
            hi: 429496729,
            mid: 2576980377,
            lo: 2576980377,
        },
        Dec12 {
            hi: 42949672,
            mid: 4123168604,
            lo: 687194767,
        },
        Dec12 {
            hi: 4294967,
            mid: 1271310319,
            lo: 2645699854,
        },
        Dec12 {
            hi: 429496,
            mid: 3133608139,
            lo: 694066715,
        },
        Dec12 {
            hi: 42949,
            mid: 2890341191,
            lo: 2216890319,
        },
        Dec12 {
            hi: 4294,
            mid: 4154504685,
            lo: 2369172679,
        },
        Dec12 {
            hi: 429,
            mid: 2133437386,
            lo: 4102387834,
        },
        Dec12 {
            hi: 42,
            mid: 4078814305,
            lo: 410238783,
        },
    ];

    // A structure that is used for faking a union of the decimal type. This allows setting mid/hi
    // with a u64, for example
    struct Dec12 {
        lo: u32,
        mid: u32,
        hi: u32,
    }

    impl Dec12 {
        const fn new(value: &Decimal) -> Self {
            Dec12 {
                lo: value.lo,
                mid: value.mid,
                hi: value.hi,
            }
        }

        // lo + mid combined
        const fn low64(&self) -> u64 {
            ((self.mid as u64) << 32) | (self.lo as u64)
        }
        fn set_low64(&mut self, value: u64) {
            self.mid = (value >> 32) as u32;
            self.lo = value as u32;
        }

        // mid + hi combined
        const fn high64(&self) -> u64 {
            ((self.hi as u64) << 32) | (self.mid as u64)
        }
        fn set_high64(&mut self, value: u64) {
            self.hi = (value >> 32) as u32;
            self.mid = value as u32;
        }

        // Returns true if successful, else false for an overflow
        fn add32(&mut self, value: u32) -> Result<(), DivError> {
            let value = value as u64;
            let new = self.low64().wrapping_add(value);
            self.set_low64(new);
            if new < value {
                self.hi = self.hi.wrapping_add(1);
                if self.hi == 0 {
                    return Err(DivError::Overflow);
                }
            }
            Ok(())
        }

        // Divide a Decimal union by a 32 bit divisor.
        // Self is overwritten with the quotient.
        // Return value is a 32 bit remainder.
        fn div32(&mut self, divisor: u32) -> u32 {
            let divisor64 = divisor as u64;
            // See if we can get by using a simple u64 division
            if self.hi != 0 {
                let mut temp = self.high64();
                let q64 = temp / divisor64;
                self.set_high64(q64);

                // Calculate the "remainder"
                temp = ((temp - q64 * divisor64) << 32) | (self.lo as u64);
                if temp == 0 {
                    return 0;
                }
                let q32 = (temp / divisor64) as u32;
                self.lo = q32;
                ((temp as u32).wrapping_sub(q32.wrapping_mul(divisor))) as u32
            } else {
                // Super easy divisor
                let low64 = self.low64();
                if low64 == 0 {
                    // Nothing to do
                    return 0;
                }
                // Do the calc
                let quotient = low64 / divisor64;
                self.set_low64(quotient);
                // Remainder is the leftover that wasn't used
                (low64.wrapping_sub(quotient.wrapping_mul(divisor64))) as u32
            }
        }

        // Divide the number by a power constant
        // Returns true if division was successful
        fn div32_const(&mut self, pow: u32) -> bool {
            let pow64 = pow as u64;
            let high64 = self.high64();
            let lo = self.lo as u64;
            let div64: u64 = high64 / pow64;
            let div = ((((high64 - div64 * pow64) << 32) + lo) / pow64) as u32;
            if self.lo == div.wrapping_mul(pow) {
                self.set_high64(div64);
                self.lo = div;
                true
            } else {
                false
            }
        }
    }

    // A structure that is used for faking a union of the decimal type with an overflow word.
    struct Dec16 {
        lo: u32,
        mid: u32,
        hi: u32,
        overflow: u32,
    }

    impl Dec16 {
        const fn zero() -> Self {
            Dec16 {
                lo: 0,
                mid: 0,
                hi: 0,
                overflow: 0,
            }
        }

        // lo + mid combined
        const fn low64(&self) -> u64 {
            ((self.mid as u64) << 32) | (self.lo as u64)
        }
        fn set_low64(&mut self, value: u64) {
            self.mid = (value >> 32) as u32;
            self.lo = value as u32;
        }

        // Equivalent to Dec12 high64 (i.e. mid + hi)
        const fn mid64(&self) -> u64 {
            ((self.hi as u64) << 32) | (self.mid as u64)
        }
        fn set_mid64(&mut self, value: u64) {
            self.hi = (value >> 32) as u32;
            self.mid = value as u32;
        }

        // hi + overflow combined
        const fn high64(&self) -> u64 {
            ((self.overflow as u64) << 32) | (self.hi as u64)
        }
        fn set_high64(&mut self, value: u64) {
            self.overflow = (value >> 32) as u32;
            self.hi = value as u32;
        }

        // Does a partial divide with a 64 bit divisor. The divisor in this case must require 64 bits
        // otherwise various assumptions fail (e.g. 32 bit quotient).
        // To assist, the upper 64 bits must be greater than the divisor for this to succeed.
        // Consequently, it will return the quotient as a 32 bit number and overwrite self with the
        // 64 bit remainder.
        fn partial_divide_64(&mut self, divisor: u64) -> u32 {
            // We make this assertion here, however below we pivot based on the data
            debug_assert!(divisor > self.mid64());

            // If we have an empty high bit, then divisor must be greater than the dividend due to
            // the assumption that the divisor REQUIRES 64 bits.
            if self.hi == 0 {
                let low64 = self.low64();
                if low64 < divisor {
                    // We can't divide at at all so result is 0. The dividend remains untouched since
                    // the full amount is the remainder.
                    return 0;
                }

                let quotient = low64 / divisor;
                self.set_low64(low64 - (quotient * divisor));
                return quotient as u32;
            }

            // Do a simple check to see if the hi portion of the dividend is greater than the hi
            // portion of the divisor.
            let divisor_hi32 = (divisor >> 32) as u32;
            if self.hi >= divisor_hi32 {
                // We know that the divisor goes into this at MOST u32::max times.
                // So we kick things off, with that assumption
                let mut low64 = self.low64();
                low64 = low64 - (divisor << 32) + divisor;
                let mut quotient = u32::max_value();

                // If we went negative then keep adding it back in
                loop {
                    if low64 < divisor {
                        break;
                    }
                    quotient -= 1;
                    low64 += divisor;
                }
                self.set_low64(low64);
                return quotient;
            }

            let mid64 = self.mid64();
            let divisor_hi32_64 = divisor_hi32 as u64;
            if mid64 < divisor_hi32_64 as u64 {
                // similar situation as above where we've got nothing left to divide
                return 0;
            }

            let mut quotient = mid64 / divisor_hi32_64;
            let mut remainder = self.lo as u64 | ((mid64 - quotient * divisor_hi32_64) << 32);

            // Do quotient * lo divisor
            let product = quotient * (divisor & 0xFFFF_FFFF);
            remainder = remainder.wrapping_sub(product);

            // Check if we've gone negative. If so, add it back
            if remainder > product.bitxor(u64::max_value()) {
                loop {
                    quotient = quotient.wrapping_sub(1);
                    remainder = remainder.wrapping_add(divisor);
                    if remainder < divisor {
                        break;
                    }
                }
            }

            self.set_low64(remainder);
            quotient as u32
        }

        // Does a partial divide with a 96 bit divisor. The divisor in this case must require 96 bits
        // otherwise various assumptions fail (e.g. 32 bit quotient).
        fn partial_divide_96(&mut self, divisor: &Dec12) -> u32 {
            let dividend = self.high64();
            let divisor_hi = divisor.hi;
            if dividend < divisor_hi as u64 {
                // Dividend is too small - entire number is remainder
                return 0;
            }

            let mut quo = (dividend / divisor_hi as u64) as u32;
            let mut remainder = (dividend as u32).wrapping_sub(quo.wrapping_mul(divisor_hi));

            // Compute full remainder
            let mut prod1 = quo as u64 * divisor.lo as u64;
            let mut prod2 = quo as u64 * divisor.mid as u64;
            prod2 += prod1 >> 32;
            prod1 = (prod1 & 0xFFFF_FFFF) | (prod2 << 32);
            prod2 >>= 32;

            let mut num = self.low64();
            num = num.wrapping_sub(prod1);
            remainder = remainder.wrapping_sub(prod2 as u32);

            // If there are carries make sure they are propogated
            if num > prod1.bitxor(u64::max_value()) {
                remainder = remainder.wrapping_sub(1);
                if remainder < (prod2 as u32).bitxor(u32::max_value()) {
                    self.set_low64(num);
                    self.hi = remainder;
                    return quo;
                }
            } else if remainder <= (prod2 as u32).bitxor(u32::max_value()) {
                self.set_low64(num);
                self.hi = remainder;
                return quo;
            }

            // Remainder went negative, add divisor back until it's positive
            prod1 = divisor.low64();
            loop {
                quo = quo.wrapping_sub(1);
                num = num.wrapping_add(prod1);
                remainder = remainder.wrapping_add(divisor_hi);

                if num < prod1 {
                    // Detected carry.
                    let tmp = remainder;
                    remainder += 1;
                    if tmp < divisor_hi {
                        break;
                    }
                }
                if remainder < divisor_hi {
                    break; // detected carry
                }
            }

            self.set_low64(num);
            self.hi = remainder;
            quo
        }
    }

    impl Decimal {
        fn low64(&self) -> u64 {
            ((self.mid as u64) << 32) | (self.lo as u64)
        }

        fn set_low64(&mut self, value: u64) {
            self.mid = (value >> 32) as u32;
            self.lo = value as u32;
        }
    }

    enum DivError {
        Overflow,
    }

    pub(crate) fn add_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        add_sub_internal(d1, d2, false)
    }

    pub(crate) fn sub_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        add_sub_internal(d1, d2, true)
    }

    fn add_sub_internal(d1: &Decimal, d2: &Decimal, sign: bool) -> CalculationResult {
        let dec1 = Dec12::new(&d1);
        let dec2 = Dec12::new(&d2);
        let xor_flags = d1.flags ^ d2.flags;
        let sign = sign ^ ((xor_flags & SIGN_MASK) != 0);

        // If the scale of the XORd flags is 0 then that indicates that the scale is the same.
        if xor_flags & SCALE_MASK == 0 {
            return aligned_add(&dec1, &dec2, d1.flags, sign);
        }

        unimplemented!("add")
    }

    fn aligned_add(d1: &Dec12, d2: &Dec12, flags: u32, sign: bool) -> CalculationResult {
        let d1_low64 = d1.low64();
        let d1_hi = d1.hi;

        // May want to consider extending Decimal
        let mut result = Decimal {
            lo: d1.lo,
            mid: d1.mid,
            hi: d1.hi,
            flags,
        };

        if sign {
            // Signs differ meaning we need to subtract
            let low64 = d1_low64.wrapping_sub(d2.low64());
            result.set_low64(low64);
            result.hi = d1_hi.wrapping_sub(d2.hi);

            // Propagate the carry. Wrapping sub would cause low64 to be greater than d1_low64
            if low64 > d1_low64 {
                result.hi -= 1;
                if result.hi >= d1_hi {
                    flip_sign(&mut result);
                }
            } else if result.hi > d1_hi {
                flip_sign(&mut result);
            }
        } else {
            // Signs are the same meaning we need to add
            let low64 = d1_low64.wrapping_add(d2.low64());
            result.set_low64(low64);
            result.hi = d1_hi.wrapping_add(d2.hi);

            // Propagate the carry. Wrapping add would cause low64 to be less than d1_low64
            if low64 < d1_low64 {
                result.hi += 1;
                if result.hi <= d1_hi {
                    // Aligned scale
                }
            } else if result.hi < d1_hi {
                // Aligned scale
            }
        }

        CalculationResult::Ok(result)
    }

    fn flip_sign(result: &mut Decimal) {
        result.flags ^= SIGN_MASK;
        result.hi = !result.hi;
        let low64 = (-(result.low64() as i64)) as u64;
        if low64 == 0 {
            result.hi += 1;
        }
        result.set_low64(low64);
    }

    pub(crate) fn div_impl(dividend: &Decimal, divisor: &Decimal) -> CalculationResult {
        if divisor.is_zero() {
            return CalculationResult::DivByZero;
        }
        if dividend.is_zero() {
            return CalculationResult::Ok(Decimal::zero());
        }

        // Pre calculate the scale and the sign
        let mut scale = (dividend.scale() as i32) - (divisor.scale() as i32);
        let sign_negative = dividend.is_sign_negative() ^ divisor.is_sign_negative();

        // Set up some variables for modification throughout
        let mut require_unscale = false;
        let mut quotient = Dec12::new(&dividend);
        let divisor = Dec12::new(&divisor);

        // Branch depending on the complexity of the divisor
        if divisor.hi | divisor.mid == 0 {
            // We have a simple(r) divisor (32 bit)
            let divisor32 = divisor.lo;

            // Remainder can only be 32 bits since the divisor is 32 bits.
            let mut remainder = quotient.div32(divisor32);
            let mut power_scale = 0;

            // Figure out how to apply the remainder (i.e. we may have performed something like 10/3 or 8/5)
            loop {
                // Remainder is 0 so we have a simple situation
                if remainder == 0 {
                    // If the scale is positive then we're actually done
                    if scale >= 0 {
                        break;
                    }
                    power_scale = 9usize.min((-scale) as usize);
                } else {
                    // We may need to normalize later, so set the flag appropriately
                    require_unscale = true;

                    // We have a remainder so we effectively want to try to adjust the quotient and add
                    // the remainder into the quotient. We do this below, however first of all we want
                    // to try to avoid overflowing so we do that check first.
                    let will_overflow = if scale == MAX_PRECISION_I32 {
                        true
                    } else {
                        // Figure out how much we can scale by
                        if let Ok(s) = find_scale(&quotient, scale) {
                            power_scale = s;
                        } else {
                            return CalculationResult::Overflow;
                        }
                        // If it comes back as 0 (i.e. 10^0 = 1) then we're going to overflow since
                        // we're doing nothing.
                        power_scale == 0
                    };
                    if will_overflow {
                        // No more scaling can be done, but remainder is non-zero so we round if necessary.
                        let tmp = remainder << 1;
                        let round = if tmp < remainder {
                            // We round if we wrapped around
                            true
                        } else {
                            if tmp >= divisor32 {
                                // If we're greater than the divisor (i.e. underflow)
                                // or if there is a lo bit set, we round
                                tmp > divisor32 || (quotient.lo & 0x1) > 0
                            } else {
                                false
                            }
                        };

                        // If we need to round, try to do so.
                        if round {
                            if let Ok(new_scale) = round_up(&mut quotient, scale) {
                                scale = new_scale;
                            } else {
                                // Overflowed
                                return CalculationResult::Overflow;
                            }
                        }
                        break;
                    }
                }

                // Do some scaling
                let power = POWERS_10[power_scale];
                scale += power_scale as i32;
                // Increase the quotient by the power that was looked up
                let overflow = increase_scale(&mut quotient, power as u64);
                if overflow > 0 {
                    return CalculationResult::Overflow;
                }

                let remainder_scaled = (remainder as u64) * (power as u64);
                let remainder_quotient = (remainder_scaled / (divisor32 as u64)) as u32;
                remainder = (remainder_scaled - remainder_quotient as u64 * divisor32 as u64) as u32;
                if let Err(DivError::Overflow) = quotient.add32(remainder_quotient) {
                    if let Ok(adj) = unscale_from_overflow(&mut quotient, scale, remainder != 0) {
                        scale = adj;
                    } else {
                        // Still overflowing
                        return CalculationResult::Overflow;
                    }
                    break;
                }
            }
        } else {
            // We have a divisor greater than 32 bits. Both of these share some quick calculation wins
            // so we'll do those before branching into separate logic.
            // The win we can do is shifting the bits to the left as much as possible. We do this to both
            // the dividend and the divisor to ensure the quotient is not changed.
            // As a simple contrived example: if we have 4 / 2 then we could bit shift all the way to the
            // left meaning that the lo portion would have nothing inside of it. Of course, shifting these
            // left one has the same result (8/4) etc.
            // The advantage is that we may be able to write off lower portions of the number making things
            // easier.
            let mut power_scale = if divisor.hi == 0 {
                divisor.mid.leading_zeros()
            } else {
                divisor.hi.leading_zeros()
            } as usize;
            let mut remainder = Dec16::zero();
            remainder.set_low64(quotient.low64() << power_scale);
            let tmp_high = ((quotient.mid as u64) + ((quotient.hi as u64) << 32)) >> (32 - power_scale);
            remainder.set_high64(tmp_high);

            // Work out the divisor after it's shifted
            let divisor64 = divisor.low64() << power_scale;
            // Check if the divisor is 64 bit or the full 96 bits
            if divisor.hi == 0 {
                // It's 64 bits
                quotient.hi = 0;

                // Calc mid/lo by shifting accordingly
                let rem_lo = remainder.lo;
                remainder.lo = remainder.mid;
                remainder.mid = remainder.hi;
                remainder.hi = remainder.overflow;
                quotient.mid = remainder.partial_divide_64(divisor64);

                remainder.hi = remainder.mid;
                remainder.mid = remainder.lo;
                remainder.lo = rem_lo;
                quotient.lo = remainder.partial_divide_64(divisor64);

                loop {
                    let rem_low64 = remainder.low64();
                    if rem_low64 == 0 {
                        // If the scale is positive then we're actually done
                        if scale >= 0 {
                            break;
                        }
                        power_scale = 9usize.min((-scale) as usize);
                    } else {
                        // We may need to normalize later, so set the flag appropriately
                        require_unscale = true;

                        // We have a remainder so we effectively want to try to adjust the quotient and add
                        // the remainder into the quotient. We do this below, however first of all we want
                        // to try to avoid overflowing so we do that check first.
                        let will_overflow = if scale == MAX_PRECISION_I32 {
                            true
                        } else {
                            // Figure out how much we can scale by
                            if let Ok(s) = find_scale(&quotient, scale) {
                                power_scale = s;
                            } else {
                                return CalculationResult::Overflow;
                            }
                            // If it comes back as 0 (i.e. 10^0 = 1) then we're going to overflow since
                            // we're doing nothing.
                            power_scale == 0
                        };
                        if will_overflow {
                            // No more scaling can be done, but remainder is non-zero so we round if necessary.
                            let mut tmp = remainder.low64();
                            let round = if (tmp as i64) < 0 {
                                // We round if we wrapped around
                                true
                            } else {
                                tmp <<= 1;
                                if tmp > divisor64 {
                                    true
                                } else {
                                    tmp == divisor64 && quotient.lo & 0x1 != 0
                                }
                            };

                            // If we need to round, try to do so.
                            if round {
                                if let Ok(new_scale) = round_up(&mut quotient, scale) {
                                    scale = new_scale;
                                } else {
                                    // Overflowed
                                    return CalculationResult::Overflow;
                                }
                            }
                            break;
                        }
                    }

                    // Do some scaling
                    let power = POWERS_10[power_scale];
                    scale += power_scale as i32;

                    // Increase the quotient by the power that was looked up
                    let overflow = increase_scale(&mut quotient, power as u64);
                    if overflow > 0 {
                        return CalculationResult::Overflow;
                    }
                    increase_scale64(&mut remainder, power as u64);

                    let tmp = remainder.partial_divide_64(divisor64);
                    if let Err(DivError::Overflow) = quotient.add32(tmp) {
                        if let Ok(adj) = unscale_from_overflow(&mut quotient, scale, remainder.low64() != 0) {
                            scale = adj;
                        } else {
                            // Still overflowing
                            return CalculationResult::Overflow;
                        }
                        break;
                    }
                }
            } else {
                // It's 96 bits
                // Start by finishing the shift left
                let divisor_mid = divisor.mid;
                let divisor_hi = divisor.hi;
                let mut divisor = divisor;
                divisor.set_low64(divisor64);
                divisor.hi = ((divisor_mid as u64 + ((divisor_hi as u64) << 32)) >> (32 - power_scale)) as u32;

                let quo = remainder.partial_divide_96(&divisor);
                quotient.set_low64(quo as u64);
                quotient.hi = 0;

                loop {
                    let mut rem_low64 = remainder.low64();
                    if rem_low64 == 0 && remainder.hi == 0 {
                        // If the scale is positive then we're actually done
                        if scale >= 0 {
                            break;
                        }
                        power_scale = 9usize.min((-scale) as usize);
                    } else {
                        // We may need to normalize later, so set the flag appropriately
                        require_unscale = true;

                        // We have a remainder so we effectively want to try to adjust the quotient and add
                        // the remainder into the quotient. We do this below, however first of all we want
                        // to try to avoid overflowing so we do that check first.
                        let will_overflow = if scale == MAX_PRECISION_I32 {
                            true
                        } else {
                            // Figure out how much we can scale by
                            if let Ok(s) = find_scale(&quotient, scale) {
                                power_scale = s;
                            } else {
                                return CalculationResult::Overflow;
                            }
                            // If it comes back as 0 (i.e. 10^0 = 1) then we're going to overflow since
                            // we're doing nothing.
                            power_scale == 0
                        };
                        if will_overflow {
                            // No more scaling can be done, but remainder is non-zero so we round if necessary.
                            let round = if (remainder.hi as i32) < 0 {
                                // We round if we wrapped around
                                true
                            } else {
                                let tmp = remainder.mid >> 31;
                                rem_low64 <<= 1;
                                remainder.set_low64(rem_low64);
                                remainder.hi = (remainder.hi << 1) + tmp;

                                if remainder.hi > divisor.hi {
                                    true
                                } else if remainder.hi == divisor.hi {
                                    let divisor_low64 = divisor.low64();
                                    if rem_low64 > divisor_low64 {
                                        true
                                    } else {
                                        rem_low64 == divisor_low64 && (quotient.lo & 1) != 0
                                    }
                                } else {
                                    false
                                }
                            };

                            // If we need to round, try to do so.
                            if round {
                                if let Ok(new_scale) = round_up(&mut quotient, scale) {
                                    scale = new_scale;
                                } else {
                                    // Overflowed
                                    return CalculationResult::Overflow;
                                }
                            }
                            break;
                        }
                    }

                    // Do some scaling
                    let power = POWERS_10[power_scale];
                    scale += power_scale as i32;

                    // Increase the quotient by the power that was looked up
                    let overflow = increase_scale(&mut quotient, power as u64);
                    if overflow > 0 {
                        return CalculationResult::Overflow;
                    }
                    let mut tmp_remainder = Dec12 {
                        lo: remainder.lo,
                        mid: remainder.mid,
                        hi: remainder.hi,
                    };
                    let overflow = increase_scale(&mut tmp_remainder, power as u64);
                    remainder.lo = tmp_remainder.lo;
                    remainder.mid = tmp_remainder.mid;
                    remainder.hi = tmp_remainder.hi;
                    remainder.overflow = overflow;

                    let tmp = remainder.partial_divide_96(&divisor);
                    if let Err(DivError::Overflow) = quotient.add32(tmp) {
                        if let Ok(adj) =
                            unscale_from_overflow(&mut quotient, scale, (remainder.low64() | remainder.high64()) != 0)
                        {
                            scale = adj;
                        } else {
                            // Still overflowing
                            return CalculationResult::Overflow;
                        }
                        break;
                    }
                }
            }
        }
        if require_unscale {
            scale = unscale(&mut quotient, scale);
        }
        CalculationResult::Ok(Decimal {
            lo: quotient.lo,
            mid: quotient.mid,
            hi: quotient.hi,
            flags: flags(sign_negative, scale as u32),
        })
    }

    pub(crate) fn mul_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        unimplemented!("mul")
    }

    pub(crate) fn rem_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
        unimplemented!("rem")
    }

    // Multiply num by power (multiple of 10). Power must be 32 bits.
    // Returns the overflow, if any
    fn increase_scale(num: &mut Dec12, power: u64) -> u32 {
        let mut tmp = (num.lo as u64) * power;
        num.lo = tmp as u32;
        tmp >>= 32;
        tmp += (num.mid as u64) * power;
        num.mid = tmp as u32;
        tmp >>= 32;
        tmp += (num.hi as u64) * power;
        num.hi = tmp as u32;
        (tmp >> 32) as u32
    }

    // Multiply num by power (multiple of 10). Power must be 32 bits.
    fn increase_scale64(num: &mut Dec16, power: u64) {
        let mut tmp = (num.lo as u64) * power;
        num.lo = tmp as u32;
        tmp >>= 32;
        tmp += (num.mid as u64) * power;
        num.set_mid64(tmp)
    }

    // Adjust the number to deal with an overflow. This function follows being scaled up (i.e. multiplied
    // by 10, so this effectively tries to reverse that by dividing by 10 then feeding in the high bit
    // to undo the overflow and rounding instead.
    // Returns the updated scale.
    fn unscale_from_overflow(num: &mut Dec12, scale: i32, sticky: bool) -> Result<i32, DivError> {
        let scale = scale - 1;
        if scale < 0 {
            return Err(DivError::Overflow);
        }

        // This function is called when the hi portion has "overflowed" upon adding one and has wrapped
        // back around to 0. Consequently, we need to "feed" that back in, but also rescaling down
        // to reverse out the overflow.
        const HIGH_BIT: u64 = 0x1_0000_0000;
        num.hi = (HIGH_BIT / 10) as u32;

        // Calc the mid
        let mut tmp = ((HIGH_BIT % 10) << 32) + (num.mid as u64);
        let mut val = (tmp / 10) as u32;
        num.mid = val;

        // Calc the lo using a similar method
        tmp = ((tmp - (val as u64) * 10) << 32) + (num.lo as u64);
        val = (tmp / 10) as u32;
        num.lo = val;

        // Work out the remainder, and round if we have one (since it doesn't fit)
        let remainder = (tmp - (val as u64) * 10) as u32;
        if remainder > 5 || (remainder == 5 && (sticky || num.lo & 0x1 > 0)) {
            let _ = num.add32(1);
        }
        Ok(scale)
    }

    // Determine the maximum value of x that ensures that the quotient when scaled up by 10^x
    // still fits in 96 bits. Ultimately, we want to make scale positive - if we can't then
    // we're going to overflow. Because x is ultimately used to lookup inside the POWERS array, it
    // must be a valid value 0 <= x <= 9
    fn find_scale(num: &Dec12, scale: i32) -> Result<usize, DivError> {
        const OVERFLOW_MAX_9_HI: u32 = 4;
        const OVERFLOW_MAX_8_HI: u32 = 42;
        const OVERFLOW_MAX_7_HI: u32 = 429;
        const OVERFLOW_MAX_6_HI: u32 = 4294;
        const OVERFLOW_MAX_5_HI: u32 = 42949;
        const OVERFLOW_MAX_4_HI: u32 = 429496;
        const OVERFLOW_MAX_3_HI: u32 = 4294967;
        const OVERFLOW_MAX_2_HI: u32 = 42949672;
        const OVERFLOW_MAX_1_HI: u32 = 429496729;
        const OVERFLOW_MAX_9_LOW64: u64 = 5441186219426131129;

        let hi = num.hi;
        let low64 = num.low64();
        let mut x = 0usize;

        // Quick check to stop us from trying to scale any more.
        //
        if hi > OVERFLOW_MAX_1_HI {
            // If it's less than 0, which it probably is - overflow. We can't do anything.
            if scale < 0 {
                return Err(DivError::Overflow);
            }
            return Ok(x);
        }

        if scale > MAX_PRECISION_I32 - 9 {
            // We can't scale by 10^9 without exceeding the max scale factor.
            // Instead, we'll try to scale by the most that we can and see if that works.
            // This is safe to do due to the check above. e.g. scale > 19 in the above, so it will
            // evaluate to 9 or less below.
            x = (MAX_PRECISION_I32 - scale) as usize;
            if hi < POWER_OVERFLOW_VALUES[x - 1].hi {
                if x as i32 + scale < 0 {
                    // We still overflow
                    return Err(DivError::Overflow);
                }
                return Ok(x);
            }
        } else if hi < OVERFLOW_MAX_9_HI || hi == OVERFLOW_MAX_9_HI && low64 <= OVERFLOW_MAX_9_LOW64 {
            return Ok(9);
        }

        // Do a binary search to find a power to scale by that is less than 9
        x = if hi > OVERFLOW_MAX_5_HI {
            if hi > OVERFLOW_MAX_3_HI {
                if hi > OVERFLOW_MAX_2_HI {
                    1
                } else {
                    2
                }
            } else {
                if hi > OVERFLOW_MAX_4_HI {
                    3
                } else {
                    4
                }
            }
        } else {
            if hi > OVERFLOW_MAX_7_HI {
                if hi > OVERFLOW_MAX_6_HI {
                    5
                } else {
                    6
                }
            } else {
                if hi > OVERFLOW_MAX_8_HI {
                    7
                } else {
                    8
                }
            }
        };

        // Double check what we've found won't overflow. Otherwise, we go one below.
        if hi == POWER_OVERFLOW_VALUES[x - 1].hi && low64 > POWER_OVERFLOW_VALUES[x - 1].low64() {
            x -= 1;
        }

        // Confirm we've actually resolved things
        if x as i32 + scale < 0 {
            Err(DivError::Overflow)
        } else {
            Ok(x)
        }
    }

    #[inline]
    fn round_up(num: &mut Dec12, scale: i32) -> Result<i32, DivError> {
        let low64 = num.low64().wrapping_add(1);
        num.set_low64(low64);
        if low64 != 0 {
            return Ok(scale);
        }
        let hi = num.hi.wrapping_add(1);
        num.hi = hi;
        if hi != 0 {
            return Ok(scale);
        }
        unscale_from_overflow(num, scale, true)
    }

    fn unscale(num: &mut Dec12, scale: i32) -> i32 {
        // Since 10 = 2 * 5, there must be a factor of 2 for every power of 10 we can extract.
        // We use this as a quick test on whether to try a given power.
        let mut scale = scale;
        while num.lo == 0 && scale >= 8 && num.div32_const(100000000) {
            scale -= 8;
        }

        if (num.lo & 0xF) == 0 && scale >= 4 && num.div32_const(10000) {
            scale -= 4;
        }

        if (num.lo & 0x3) == 0 && scale >= 2 && num.div32_const(100) {
            scale -= 2;
        }

        if (num.lo & 0x1) == 0 && scale >= 1 && num.div32_const(10) {
            scale -= 1;
        }
        scale
    }
}

// Returns remainder
pub(crate) fn div_by_u32(bits: &mut [u32], divisor: u32) -> u32 {
    if divisor == 0 {
        // Divide by zero
        panic!("Internal error: divide by zero");
    } else if divisor == 1 {
        // dividend remains unchanged
        0
    } else {
        let mut remainder = 0u32;
        let divisor = u64::from(divisor);
        for part in bits.iter_mut().rev() {
            let temp = (u64::from(remainder) << 32) + u64::from(*part);
            remainder = (temp % divisor) as u32;
            *part = (temp / divisor) as u32;
        }

        remainder
    }
}

fn div_by_10(bits: &mut [u32; 3]) -> u32 {
    let mut remainder = 0u32;
    let divisor = 10u64;
    for part in bits.iter_mut().rev() {
        let temp = (u64::from(remainder) << 32) + u64::from(*part);
        remainder = (temp % divisor) as u32;
        *part = (temp / divisor) as u32;
    }

    remainder
}

#[inline]
fn shl1_internal(bits: &mut [u32], carry: u32) -> u32 {
    let mut carry = carry;
    for part in bits.iter_mut() {
        let b = *part >> 31;
        *part = (*part << 1) | carry;
        carry = b;
    }
    carry
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

#[inline]
pub(crate) fn is_all_zero(bits: &[u32]) -> bool {
    bits.iter().all(|b| *b == 0)
}

macro_rules! impl_from {
    ($T:ty, $from_ty:path) => {
        impl core::convert::From<$T> for Decimal {
            #[inline]
            fn from(t: $T) -> Self {
                $from_ty(t).unwrap()
            }
        }
    };
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

#[cfg(has_i128)]
impl_from!(i128, FromPrimitive::from_i28);
#[cfg(has_i128)]
impl_from!(u128, FromPrimitive::from_u28);

macro_rules! forward_val_val_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self).$method(&other)
            }
        }
    };
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
    };
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
    };
}

macro_rules! forward_all_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        forward_val_val_binop!(impl $imp for $res, $method);
        forward_ref_val_binop!(impl $imp for $res, $method);
        forward_val_ref_binop!(impl $imp for $res, $method);
    };
}

impl Zero for Decimal {
    fn zero() -> Decimal {
        Decimal {
            flags: 0,
            hi: 0,
            lo: 0,
            mid: 0,
        }
    }

    fn is_zero(&self) -> bool {
        self.lo.is_zero() && self.mid.is_zero() && self.hi.is_zero()
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

impl Signed for Decimal {
    fn abs(&self) -> Self {
        self.abs()
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Decimal::zero()
        } else {
            self.abs()
        }
    }

    fn signum(&self) -> Self {
        if self.is_zero() {
            Decimal::zero()
        } else {
            let mut value = Decimal::one();
            if self.is_sign_negative() {
                value.set_sign_negative(true);
            }
            value
        }
    }

    fn is_positive(&self) -> bool {
        self.is_sign_positive()
    }

    fn is_negative(&self) -> bool {
        self.is_sign_negative()
    }
}

// dedicated implementation for the most common case.
fn parse_str_radix_10(str: &str) -> Result<Decimal, crate::Error> {
    if str.is_empty() {
        return Err(Error::new("Invalid decimal: empty"));
    }

    let mut offset = 0;
    let mut len = str.len();
    let bytes = str.as_bytes();
    let mut negative = false; // assume positive

    // handle the sign
    if bytes[offset] == b'-' {
        negative = true; // leading minus means negative
        offset += 1;
        len -= 1;
    } else if bytes[offset] == b'+' {
        // leading + allowed
        offset += 1;
        len -= 1;
    }

    // should now be at numeric part of the significand
    let mut digits_before_dot: i32 = -1; // digits before '.', -1 if no '.'
    let mut coeff = ArrayVec::<[_; MAX_STR_BUFFER_SIZE]>::new(); // integer significand array

    let mut maybe_round = false;
    while len > 0 {
        let b = bytes[offset];
        match b {
            b'0'..=b'9' => {
                coeff.push(u32::from(b - b'0'));
                offset += 1;
                len -= 1;

                // If the coefficient is longer than the max, exit early
                if coeff.len() as u32 > 28 {
                    maybe_round = true;
                    break;
                }
            }
            b'.' => {
                if digits_before_dot >= 0 {
                    return Err(Error::new("Invalid decimal: two decimal points"));
                }
                digits_before_dot = coeff.len() as i32;
                offset += 1;
                len -= 1;
            }
            b'_' => {
                // Must start with a number...
                if coeff.is_empty() {
                    return Err(Error::new("Invalid decimal: must start lead with a number"));
                }
                offset += 1;
                len -= 1;
            }
            _ => return Err(Error::new("Invalid decimal: unknown character")),
        }
    }

    // If we exited before the end of the string then do some rounding if necessary
    if maybe_round && offset < bytes.len() {
        let next_byte = bytes[offset];
        let digit = match next_byte {
            b'0'..=b'9' => u32::from(next_byte - b'0'),
            b'_' => 0,
            b'.' => {
                // Still an error if we have a second dp
                if digits_before_dot >= 0 {
                    return Err(Error::new("Invalid decimal: two decimal points"));
                }
                0
            }
            _ => return Err(Error::new("Invalid decimal: unknown character")),
        };

        // Round at midpoint
        if digit >= 5 {
            let mut index = coeff.len() - 1;
            loop {
                let new_digit = coeff[index] + 1;
                if new_digit <= 9 {
                    coeff[index] = new_digit;
                    break;
                } else {
                    coeff[index] = 0;
                    if index == 0 {
                        coeff.insert(0, 1u32);
                        digits_before_dot += 1;
                        coeff.pop();
                        break;
                    }
                }
                index -= 1;
            }
        }
    }

    // here when no characters left
    if coeff.is_empty() {
        return Err(Error::new("Invalid decimal: no digits found"));
    }

    let mut scale = if digits_before_dot >= 0 {
        // we had a decimal place so set the scale
        (coeff.len() as u32) - (digits_before_dot as u32)
    } else {
        0
    };

    let mut data = [0u32, 0u32, 0u32];
    let mut tmp = [0u32, 0u32, 0u32];
    let len = coeff.len();
    for (i, digit) in coeff.iter().enumerate() {
        // If the data is going to overflow then we should go into recovery mode
        tmp[0] = data[0];
        tmp[1] = data[1];
        tmp[2] = data[2];
        let overflow = mul_by_10(&mut tmp);
        if overflow > 0 {
            // This means that we have more data to process, that we're not sure what to do with.
            // This may or may not be an issue - depending on whether we're past a decimal point
            // or not.
            if (i as i32) < digits_before_dot && i + 1 < len {
                return Err(Error::new("Invalid decimal: overflow from too many digits"));
            }

            if *digit >= 5 {
                let carry = add_one_internal(&mut data);
                if carry > 0 {
                    // Highly unlikely scenario which is more indicative of a bug
                    return Err(Error::new("Invalid decimal: overflow when rounding"));
                }
            }
            // We're also one less digit so reduce the scale
            let diff = (len - i) as u32;
            if diff > scale {
                return Err(Error::new("Invalid decimal: overflow from scale mismatch"));
            }
            scale -= diff;
            break;
        } else {
            data[0] = tmp[0];
            data[1] = tmp[1];
            data[2] = tmp[2];
            let carry = add_by_internal(&mut data, &[*digit]);
            if carry > 0 {
                // Highly unlikely scenario which is more indicative of a bug
                return Err(Error::new("Invalid decimal: overflow from carry"));
            }
        }
    }

    Ok(Decimal {
        lo: data[0],
        mid: data[1],
        hi: data[2],
        flags: flags(negative, scale),
    })
}

pub fn parse_str_radix_n(str: &str, radix: u32) -> Result<Decimal, crate::Error> {
    if str.is_empty() {
        return Err(Error::new("Invalid decimal: empty"));
    }
    if radix < 2 {
        return Err(Error::new("Unsupported radix < 2"));
    }
    if radix > 36 {
        // As per trait documentation
        return Err(Error::new("Unsupported radix > 36"));
    }

    let mut offset = 0;
    let mut len = str.len();
    let bytes = str.as_bytes();
    let mut negative = false; // assume positive

    // handle the sign
    if bytes[offset] == b'-' {
        negative = true; // leading minus means negative
        offset += 1;
        len -= 1;
    } else if bytes[offset] == b'+' {
        // leading + allowed
        offset += 1;
        len -= 1;
    }

    // should now be at numeric part of the significand
    let mut digits_before_dot: i32 = -1; // digits before '.', -1 if no '.'
    let mut coeff = ArrayVec::<[_; 96]>::new(); // integer significand array

    // Supporting different radix
    let (max_n, max_alpha_lower, max_alpha_upper) = if radix <= 10 {
        (b'0' + (radix - 1) as u8, 0, 0)
    } else {
        let adj = (radix - 11) as u8;
        (b'9', adj + b'a', adj + b'A')
    };

    // Estimate the max precision. All in all, it needs to fit into 96 bits.
    // Rather than try to estimate, I've included the constants directly in here. We could,
    // perhaps, replace this with a formula if it's faster - though it does appear to be log2.
    let estimated_max_precision = match radix {
        2 => 96,
        3 => 61,
        4 => 48,
        5 => 42,
        6 => 38,
        7 => 35,
        8 => 32,
        9 => 31,
        10 => 28,
        11 => 28,
        12 => 27,
        13 => 26,
        14 => 26,
        15 => 25,
        16 => 24,
        17 => 24,
        18 => 24,
        19 => 23,
        20 => 23,
        21 => 22,
        22 => 22,
        23 => 22,
        24 => 21,
        25 => 21,
        26 => 21,
        27 => 21,
        28 => 20,
        29 => 20,
        30 => 20,
        31 => 20,
        32 => 20,
        33 => 20,
        34 => 19,
        35 => 19,
        36 => 19,
        _ => return Err(Error::new("Unsupported radix")),
    };

    let mut maybe_round = false;
    while len > 0 {
        let b = bytes[offset];
        match b {
            b'0'..=b'9' => {
                if b > max_n {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                coeff.push(u32::from(b - b'0'));
                offset += 1;
                len -= 1;

                // If the coefficient is longer than the max, exit early
                if coeff.len() as u32 > estimated_max_precision {
                    maybe_round = true;
                    break;
                }
            }
            b'a'..=b'z' => {
                if b > max_alpha_lower {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                coeff.push(u32::from(b - b'a') + 10);
                offset += 1;
                len -= 1;

                if coeff.len() as u32 > estimated_max_precision {
                    maybe_round = true;
                    break;
                }
            }
            b'A'..=b'Z' => {
                if b > max_alpha_upper {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                coeff.push(u32::from(b - b'A') + 10);
                offset += 1;
                len -= 1;

                if coeff.len() as u32 > estimated_max_precision {
                    maybe_round = true;
                    break;
                }
            }
            b'.' => {
                if digits_before_dot >= 0 {
                    return Err(Error::new("Invalid decimal: two decimal points"));
                }
                digits_before_dot = coeff.len() as i32;
                offset += 1;
                len -= 1;
            }
            b'_' => {
                // Must start with a number...
                if coeff.is_empty() {
                    return Err(Error::new("Invalid decimal: must start lead with a number"));
                }
                offset += 1;
                len -= 1;
            }
            _ => return Err(Error::new("Invalid decimal: unknown character")),
        }
    }

    // If we exited before the end of the string then do some rounding if necessary
    if maybe_round && offset < bytes.len() {
        let next_byte = bytes[offset];
        let digit = match next_byte {
            b'0'..=b'9' => {
                if next_byte > max_n {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                u32::from(next_byte - b'0')
            }
            b'a'..=b'z' => {
                if next_byte > max_alpha_lower {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                u32::from(next_byte - b'a') + 10
            }
            b'A'..=b'Z' => {
                if next_byte > max_alpha_upper {
                    return Err(Error::new("Invalid decimal: invalid character"));
                }
                u32::from(next_byte - b'A') + 10
            }
            b'_' => 0,
            b'.' => {
                // Still an error if we have a second dp
                if digits_before_dot >= 0 {
                    return Err(Error::new("Invalid decimal: two decimal points"));
                }
                0
            }
            _ => return Err(Error::new("Invalid decimal: unknown character")),
        };

        // Round at midpoint
        let midpoint = if radix & 0x1 == 1 { radix / 2 } else { (radix + 1) / 2 };
        if digit >= midpoint {
            let mut index = coeff.len() - 1;
            loop {
                let new_digit = coeff[index] + 1;
                if new_digit <= 9 {
                    coeff[index] = new_digit;
                    break;
                } else {
                    coeff[index] = 0;
                    if index == 0 {
                        coeff.insert(0, 1u32);
                        digits_before_dot += 1;
                        coeff.pop();
                        break;
                    }
                }
                index -= 1;
            }
        }
    }

    // here when no characters left
    if coeff.is_empty() {
        return Err(Error::new("Invalid decimal: no digits found"));
    }

    let mut scale = if digits_before_dot >= 0 {
        // we had a decimal place so set the scale
        (coeff.len() as u32) - (digits_before_dot as u32)
    } else {
        0
    };

    // Parse this using specified radix
    let mut data = [0u32, 0u32, 0u32];
    let mut tmp = [0u32, 0u32, 0u32];
    let len = coeff.len();
    for (i, digit) in coeff.iter().enumerate() {
        // If the data is going to overflow then we should go into recovery mode
        tmp[0] = data[0];
        tmp[1] = data[1];
        tmp[2] = data[2];
        let overflow = mul_by_u32(&mut tmp, radix);
        if overflow > 0 {
            // This means that we have more data to process, that we're not sure what to do with.
            // This may or may not be an issue - depending on whether we're past a decimal point
            // or not.
            if (i as i32) < digits_before_dot && i + 1 < len {
                return Err(Error::new("Invalid decimal: overflow from too many digits"));
            }

            if *digit >= 5 {
                let carry = add_one_internal(&mut data);
                if carry > 0 {
                    // Highly unlikely scenario which is more indicative of a bug
                    return Err(Error::new("Invalid decimal: overflow when rounding"));
                }
            }
            // We're also one less digit so reduce the scale
            let diff = (len - i) as u32;
            if diff > scale {
                return Err(Error::new("Invalid decimal: overflow from scale mismatch"));
            }
            scale -= diff;
            break;
        } else {
            data[0] = tmp[0];
            data[1] = tmp[1];
            data[2] = tmp[2];
            let carry = add_by_internal(&mut data, &[*digit]);
            if carry > 0 {
                // Highly unlikely scenario which is more indicative of a bug
                return Err(Error::new("Invalid decimal: overflow from carry"));
            }
        }
    }

    Ok(Decimal {
        lo: data[0],
        mid: data[1],
        hi: data[2],
        flags: flags(negative, scale),
    })
}

impl Num for Decimal {
    type FromStrRadixErr = Error;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Decimal::from_str_radix(str, radix)
    }
}

impl FromStr for Decimal {
    type Err = Error;

    fn from_str(value: &str) -> Result<Decimal, Self::Err> {
        parse_str_radix_10(value)
    }
}

impl FromPrimitive for Decimal {
    fn from_i32(n: i32) -> Option<Decimal> {
        let flags: u32;
        let value_copy: i64;
        if n >= 0 {
            flags = 0;
            value_copy = n as i64;
        } else {
            flags = SIGN_MASK;
            value_copy = -(n as i64);
        }
        Some(Decimal {
            flags,
            lo: value_copy as u32,
            mid: 0,
            hi: 0,
        })
    }

    fn from_i64(n: i64) -> Option<Decimal> {
        let flags: u32;
        let value_copy: i128;
        if n >= 0 {
            flags = 0;
            value_copy = n as i128;
        } else {
            flags = SIGN_MASK;
            value_copy = -(n as i128);
        }
        Some(Decimal {
            flags,
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
                zero.set_sign_negative(true);
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
                zero.set_sign_negative(true);
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
    fn to_i64(&self) -> Option<i64> {
        let d = self.trunc();
        // Quick overflow check
        if d.hi != 0 || (d.mid & 0x8000_0000) > 0 {
            // Overflow
            return None;
        }

        let raw: i64 = (i64::from(d.mid) << 32) | i64::from(d.lo);
        if self.is_sign_negative() {
            Some(-raw)
        } else {
            Some(raw)
        }
    }

    fn to_i128(&self) -> Option<i128> {
        let d = self.trunc();
        let raw: i128 = ((i128::from(d.hi) << 64) | i128::from(d.mid) << 32) | i128::from(d.lo);
        if self.is_sign_negative() {
            Some(-raw)
        } else {
            Some(raw)
        }
    }

    fn to_u64(&self) -> Option<u64> {
        if self.is_sign_negative() {
            return None;
        }

        let d = self.trunc();
        if d.hi != 0 {
            // Overflow
            return None;
        }

        Some((u64::from(d.mid) << 32) | u64::from(d.lo))
    }

    fn to_u128(&self) -> Option<u128> {
        if self.is_sign_negative() {
            return None;
        }

        let d = self.trunc();
        Some((u128::from(d.hi) << 64) | (u128::from(d.mid) << 32) | u128::from(d.lo))
    }

    fn to_f64(&self) -> Option<f64> {
        if self.scale() == 0 {
            let integer = self.to_i64();
            match integer {
                Some(i) => Some(i as f64),
                None => None,
            }
        } else {
            let sign: f64 = if self.is_sign_negative() { -1.0 } else { 1.0 };
            let mut mantissa: u128 = self.lo.into();
            mantissa |= (self.mid as u128) << 32;
            mantissa |= (self.hi as u128) << 64;
            // scale is at most 28, so this fits comfortably into a u128.
            let scale = self.scale();
            let precision: u128 = 10_u128.pow(scale);
            let integral_part = mantissa / precision;
            let frac_part = mantissa % precision;
            let frac_f64 = (frac_part as f64) / (precision as f64);
            let value = sign * ((integral_part as f64) + frac_f64);
            let round_to = 10f64.powi(self.scale() as i32);
            Some(value * round_to / round_to)
        }
    }
}

impl core::convert::TryFrom<f32> for Decimal {
    type Error = crate::Error;

    fn try_from(value: f32) -> Result<Self, Error> {
        Self::from_f32(value).ok_or_else(|| Error::new("Failed to convert to Decimal"))
    }
}

impl core::convert::TryFrom<f64> for Decimal {
    type Error = crate::Error;

    fn try_from(value: f64) -> Result<Self, Error> {
        Self::from_f64(value).ok_or_else(|| Error::new("Failed to convert to Decimal"))
    }
}

impl core::convert::TryFrom<Decimal> for f32 {
    type Error = crate::Error;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Decimal::to_f32(&value).ok_or_else(|| Error::new("Failed to convert to f32"))
    }
}

impl core::convert::TryFrom<Decimal> for f64 {
    type Error = crate::Error;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        Decimal::to_f64(&value).ok_or_else(|| Error::new("Failed to convert to f64"))
    }
}

// impl that doesn't allocate for serialization purposes.
pub(crate) fn to_str_internal(
    value: &Decimal,
    append_sign: bool,
    precision: Option<usize>,
) -> ArrayString<[u8; MAX_STR_BUFFER_SIZE]> {
    // Get the scale - where we need to put the decimal point
    let scale = value.scale() as usize;

    // Convert to a string and manipulate that (neg at front, inject decimal)
    let mut chars = ArrayVec::<[_; MAX_STR_BUFFER_SIZE]>::new();
    let mut working = [value.lo, value.mid, value.hi];
    while !is_all_zero(&working) {
        let remainder = div_by_u32(&mut working, 10u32);
        chars.push(char::from(b'0' + remainder as u8));
    }
    while scale > chars.len() {
        chars.push('0');
    }

    let prec = match precision {
        Some(prec) => prec,
        None => scale,
    };

    let len = chars.len();
    let whole_len = len - scale;
    let mut rep = ArrayString::new();
    if append_sign && value.is_sign_negative() {
        rep.push('-');
    }
    for i in 0..whole_len + prec {
        if i == len - scale {
            if i == 0 {
                rep.push('0');
            }
            rep.push('.');
        }

        if i >= len {
            rep.push('0');
        } else {
            let c = chars[len - i - 1];
            rep.push(c);
        }
    }

    // corner case for when we truncated everything in a low fractional
    if rep.is_empty() {
        rep.push('0');
    }

    rep
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let rep = to_str_internal(self, false, f.precision());
        f.pad_integral(self.is_sign_positive(), "", rep.as_str())
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

fn fmt_scientific_notation(value: &Decimal, exponent_symbol: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    // Get the scale - this is the e value. With multiples of 10 this may get bigger.
    let mut exponent = -(value.scale() as isize);

    // Convert the integral to a string
    let mut chars = Vec::new();
    let mut working = [value.lo, value.mid, value.hi];
    while !is_all_zero(&working) {
        let remainder = div_by_u32(&mut working, 10u32);
        chars.push(char::from(b'0' + remainder as u8));
    }

    // First of all, apply scientific notation rules. That is:
    //  1. If non-zero digit comes first, move decimal point left so that e is a positive integer
    //  2. If decimal point comes first, move decimal point right until after the first non-zero digit
    // Since decimal notation naturally lends itself this way, we just need to inject the decimal
    // point in the right place and adjust the exponent accordingly.

    let len = chars.len();
    let mut rep;
    if len > 1 {
        if chars.iter().take(len - 1).all(|c| *c == '0') {
            // Chomp off the zero's.
            rep = chars.iter().skip(len - 1).collect::<String>();
        } else {
            chars.insert(len - 1, '.');
            rep = chars.iter().rev().collect::<String>();
        }
        exponent += (len - 1) as isize;
    } else {
        rep = chars.iter().collect::<String>();
    }

    rep.push_str(exponent_symbol);
    rep.push_str(&exponent.to_string());
    f.pad_integral(value.is_sign_positive(), "", &rep)
}

impl fmt::LowerExp for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_scientific_notation(self, "e", f)
    }
}

impl fmt::UpperExp for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_scientific_notation(self, "E", f)
    }
}

impl Neg for Decimal {
    type Output = Decimal;

    fn neg(self) -> Decimal {
        let mut copy = self;
        copy.set_sign_negative(self.is_sign_positive());
        copy
    }
}

impl<'a> Neg for &'a Decimal {
    type Output = Decimal;

    fn neg(self) -> Decimal {
        Decimal {
            flags: flags(!self.is_sign_negative(), self.scale()),
            hi: self.hi,
            lo: self.lo,
            mid: self.mid,
        }
    }
}

forward_all_binop!(impl Add for Decimal, add);

impl<'a, 'b> Add<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn add(self, other: &Decimal) -> Decimal {
        match self.checked_add(*other) {
            Some(sum) => sum,
            None => panic!("Addition overflowed"),
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

impl<'a> AddAssign<&'a Decimal> for Decimal {
    fn add_assign(&mut self, other: &'a Decimal) {
        Decimal::add_assign(self, *other)
    }
}

impl<'a> AddAssign<Decimal> for &'a mut Decimal {
    fn add_assign(&mut self, other: Decimal) {
        Decimal::add_assign(*self, other)
    }
}

impl<'a> AddAssign<&'a Decimal> for &'a mut Decimal {
    fn add_assign(&mut self, other: &'a Decimal) {
        Decimal::add_assign(*self, *other)
    }
}

forward_all_binop!(impl Sub for Decimal, sub);

impl<'a, 'b> Sub<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline(always)]
    fn sub(self, other: &Decimal) -> Decimal {
        match self.checked_sub(*other) {
            Some(diff) => diff,
            None => panic!("Subtraction overflowed"),
        }
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

impl<'a> SubAssign<&'a Decimal> for Decimal {
    fn sub_assign(&mut self, other: &'a Decimal) {
        Decimal::sub_assign(self, *other)
    }
}

impl<'a> SubAssign<Decimal> for &'a mut Decimal {
    fn sub_assign(&mut self, other: Decimal) {
        Decimal::sub_assign(*self, other)
    }
}

impl<'a> SubAssign<&'a Decimal> for &'a mut Decimal {
    fn sub_assign(&mut self, other: &'a Decimal) {
        Decimal::sub_assign(*self, *other)
    }
}

forward_all_binop!(impl Mul for Decimal, mul);

impl<'a, 'b> Mul<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn mul(self, other: &Decimal) -> Decimal {
        match self.checked_mul(*other) {
            Some(prod) => prod,
            None => panic!("Multiplication overflowed"),
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

impl<'a> MulAssign<&'a Decimal> for Decimal {
    fn mul_assign(&mut self, other: &'a Decimal) {
        Decimal::mul_assign(self, *other)
    }
}

impl<'a> MulAssign<Decimal> for &'a mut Decimal {
    fn mul_assign(&mut self, other: Decimal) {
        Decimal::mul_assign(*self, other)
    }
}

impl<'a> MulAssign<&'a Decimal> for &'a mut Decimal {
    fn mul_assign(&mut self, other: &'a Decimal) {
        Decimal::mul_assign(*self, *other)
    }
}

forward_all_binop!(impl Div for Decimal, div);

impl<'a, 'b> Div<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    fn div(self, other: &Decimal) -> Decimal {
        match ops::div_impl(&self, other) {
            CalculationResult::Ok(quot) => quot,
            CalculationResult::Overflow => panic!("Division overflowed"),
            CalculationResult::DivByZero => panic!("Division by zero"),
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

impl<'a> DivAssign<&'a Decimal> for Decimal {
    fn div_assign(&mut self, other: &'a Decimal) {
        Decimal::div_assign(self, *other)
    }
}

impl<'a> DivAssign<Decimal> for &'a mut Decimal {
    fn div_assign(&mut self, other: Decimal) {
        Decimal::div_assign(*self, other)
    }
}

impl<'a> DivAssign<&'a Decimal> for &'a mut Decimal {
    fn div_assign(&mut self, other: &'a Decimal) {
        Decimal::div_assign(*self, *other)
    }
}

forward_all_binop!(impl Rem for Decimal, rem);

impl<'a, 'b> Rem<&'b Decimal> for &'a Decimal {
    type Output = Decimal;

    #[inline]
    fn rem(self, other: &Decimal) -> Decimal {
        match self.checked_rem(*other) {
            Some(rem) => rem,
            None => panic!("Division by zero"),
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

impl<'a> RemAssign<&'a Decimal> for Decimal {
    fn rem_assign(&mut self, other: &'a Decimal) {
        Decimal::rem_assign(self, *other)
    }
}

impl<'a> RemAssign<Decimal> for &'a mut Decimal {
    fn rem_assign(&mut self, other: Decimal) {
        Decimal::rem_assign(*self, other)
    }
}

impl<'a> RemAssign<&'a Decimal> for &'a mut Decimal {
    fn rem_assign(&mut self, other: &'a Decimal) {
        Decimal::rem_assign(*self, *other)
    }
}

impl PartialEq for Decimal {
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        self.cmp(other) == Equal
    }
}

impl Eq for Decimal {}

impl Hash for Decimal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let n = self.normalize();
        n.lo.hash(state);
        n.mid.hash(state);
        n.hi.hash(state);
        n.flags.hash(state);
    }
}

impl PartialOrd for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Decimal) -> Ordering {
        // Quick exit if major differences
        if self.is_zero() && other.is_zero() {
            return Ordering::Equal;
        }
        let self_negative = self.is_sign_negative();
        let other_negative = other.is_sign_negative();
        if self_negative && !other_negative {
            return Ordering::Less;
        } else if !self_negative && other_negative {
            return Ordering::Greater;
        }

        // If we have 1.23 and 1.2345 then we have
        //  123 scale 2 and 12345 scale 4
        //  We need to convert the first to
        //  12300 scale 4 so we can compare equally
        let left: &Decimal;
        let right: &Decimal;
        if self_negative && other_negative {
            // Both are negative, so reverse cmp
            left = other;
            right = self;
        } else {
            left = self;
            right = other;
        }
        let mut left_scale = left.scale();
        let mut right_scale = right.scale();

        if left_scale == right_scale {
            // Fast path for same scale
            if left.hi != right.hi {
                return left.hi.cmp(&right.hi);
            }
            if left.mid != right.mid {
                return left.mid.cmp(&right.mid);
            }
            return left.lo.cmp(&right.lo);
        }

        // Rescale and compare
        let mut left_raw = [left.lo, left.mid, left.hi];
        let mut right_raw = [right.lo, right.mid, right.hi];
        rescale_to_maximum_scale(&mut left_raw, &mut left_scale, &mut right_raw, &mut right_scale);
        cmp_internal(&left_raw, &right_raw)
    }
}

impl Sum for Decimal {
    fn sum<I: Iterator<Item = Decimal>>(iter: I) -> Self {
        let mut sum = Decimal::zero();
        for i in iter {
            sum += i;
        }
        sum
    }
}

impl<'a> Sum<&'a Decimal> for Decimal {
    fn sum<I: Iterator<Item = &'a Decimal>>(iter: I) -> Self {
        let mut sum = Decimal::zero();
        for i in iter {
            sum += i;
        }
        sum
    }
}

#[cfg(test)]
mod test {
    // Tests on private methods.
    //
    // All public tests should go under `tests/`.

    use super::*;

    #[test]
    fn it_can_rescale_to_maximum_scale() {
        fn extract(value: &str) -> ([u32; 3], u32) {
            let v = Decimal::from_str(value).unwrap();
            ([v.lo, v.mid, v.hi], v.scale())
        }

        let tests = &[
            ("1", "1", "1", "1"),
            ("1", "1.0", "1.0", "1.0"),
            ("1", "1.00000", "1.00000", "1.00000"),
            ("1", "1.0000000000", "1.0000000000", "1.0000000000"),
            (
                "1",
                "1.00000000000000000000",
                "1.00000000000000000000",
                "1.00000000000000000000",
            ),
            ("1.1", "1.1", "1.1", "1.1"),
            ("1.1", "1.10000", "1.10000", "1.10000"),
            ("1.1", "1.1000000000", "1.1000000000", "1.1000000000"),
            (
                "1.1",
                "1.10000000000000000000",
                "1.10000000000000000000",
                "1.10000000000000000000",
            ),
            (
                "0.6386554621848739495798319328",
                "11.815126050420168067226890757",
                "0.638655462184873949579831933",
                "11.815126050420168067226890757",
            ),
            (
                "0.0872727272727272727272727272", // Scale 28
                "843.65000000",                   // Scale 8
                "0.0872727272727272727272727",    // 25
                "843.6500000000000000000000000",  // 25
            ),
        ];

        for &(left_raw, right_raw, expected_left, expected_right) in tests {
            // Left = the value to rescale
            // Right = the new scale we're scaling to
            // Expected = the expected left value after rescale
            let (expected_left, expected_lscale) = extract(expected_left);
            let (expected_right, expected_rscale) = extract(expected_right);

            let (mut left, mut left_scale) = extract(left_raw);
            let (mut right, mut right_scale) = extract(right_raw);
            rescale_to_maximum_scale(&mut left, &mut left_scale, &mut right, &mut right_scale);
            assert_eq!(left, expected_left);
            assert_eq!(left_scale, expected_lscale);
            assert_eq!(right, expected_right);
            assert_eq!(right_scale, expected_rscale);

            // Also test the transitive case
            let (mut left, mut left_scale) = extract(left_raw);
            let (mut right, mut right_scale) = extract(right_raw);
            rescale_to_maximum_scale(&mut right, &mut right_scale, &mut left, &mut left_scale);
            assert_eq!(left, expected_left);
            assert_eq!(left_scale, expected_lscale);
            assert_eq!(right, expected_right);
            assert_eq!(right_scale, expected_rscale);
        }
    }

    #[test]
    fn it_can_rescale_internal() {
        fn extract(value: &str) -> ([u32; 3], u32) {
            let v = Decimal::from_str(value).unwrap();
            ([v.lo, v.mid, v.hi], v.scale())
        }

        let tests = &[
            ("1", 0, "1"),
            ("1", 1, "1.0"),
            ("1", 5, "1.00000"),
            ("1", 10, "1.0000000000"),
            ("1", 20, "1.00000000000000000000"),
            ("0.6386554621848739495798319328", 27, "0.638655462184873949579831933"),
            (
                "843.65000000",                  // Scale 8
                25,                              // 25
                "843.6500000000000000000000000", // 25
            ),
            (
                "843.65000000",                     // Scale 8
                30,                                 // 30
                "843.6500000000000000000000000000", // 28
            ),
        ];

        for &(value_raw, new_scale, expected_value) in tests {
            let (expected_value, _) = extract(expected_value);
            let (mut value, mut value_scale) = extract(value_raw);
            rescale_internal(&mut value, &mut value_scale, new_scale);
            assert_eq!(value, expected_value);
        }
    }

    #[test]
    fn test_shl1_internal() {
        struct TestCase {
            // One thing to be cautious of is that the structure of a number here for shifting left is
            // the reverse of how you may conceive this mentally. i.e. a[2] contains the higher order
            // bits: a[2] a[1] a[0]
            given: [u32; 3],
            given_carry: u32,
            expected: [u32; 3],
            expected_carry: u32,
        }
        let tests = [
            TestCase {
                given: [1, 0, 0],
                given_carry: 0,
                expected: [2, 0, 0],
                expected_carry: 0,
            },
            TestCase {
                given: [1, 0, 2147483648],
                given_carry: 1,
                expected: [3, 0, 0],
                expected_carry: 1,
            },
        ];
        for case in &tests {
            let mut test = [case.given[0], case.given[1], case.given[2]];
            let carry = shl1_internal(&mut test, case.given_carry);
            assert_eq!(
                test, case.expected,
                "Bits: {:?} << 1 | {}",
                case.given, case.given_carry
            );
            assert_eq!(
                carry, case.expected_carry,
                "Carry: {:?} << 1 | {}",
                case.given, case.given_carry
            )
        }
    }
}
