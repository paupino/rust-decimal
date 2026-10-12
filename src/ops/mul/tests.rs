// Tests for the helpers behind `mul_impl`, checked against native integer arithmetic. The products
// themselves are tested against .NET `System.Decimal` results in `tests/generated/Mul_*.csv`.

use super::*;
use crate::decimal::CalculationResult;

const MANTISSA_MAX: u128 = (1u128 << 96) - 1;

/// xorshift64* - small, deterministic and good enough for test case generation.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn range(&mut self, lo: u64, hi_inclusive: u64) -> u64 {
        lo + self.below(hi_inclusive - lo + 1)
    }

    fn bool(&mut self) -> bool {
        self.next() & 1 == 1
    }

    fn u128(&mut self) -> u128 {
        ((self.next() as u128) << 64) | self.next() as u128
    }

    /// A value with exactly `bits` significant bits (0 gives 0).
    fn bits(&mut self, bits: u32) -> u128 {
        if bits == 0 {
            return 0;
        }
        let v = self.u128() >> (128 - bits);
        v | (1u128 << (bits - 1))
    }

    /// A value whose bit length is chosen uniformly from `lo..=hi`.
    fn bits_in(&mut self, lo: u64, hi: u64) -> u128 {
        let bits = self.range(lo, hi) as u32;
        self.bits(bits)
    }
}

fn pow10(k: u32) -> u128 {
    10u128.pow(k)
}

#[test]
fn mul_with_scale_above_max_does_not_panic() {
    // Operands with a scale above 28 can't come from the checked constructors or a string, so they
    // aren't in the CSVs, but they can be built through `bytemuck`.
    let mut rng = Rng(0x6A09_E667_F3BC_C909);
    let raw = |m: u128, negative: bool, scale: u32| {
        Decimal::from_parts_raw_unchecked(
            m as u32,
            (m >> 32) as u32,
            (m >> 64) as u32,
            crate::decimal::flags(negative, scale),
        )
    };
    for _ in 0..10_000 {
        let a = raw(rng.bits_in(0, 96), rng.bool(), rng.range(0, 255) as u32);
        let b = raw(
            rng.bits_in(0, 96),
            rng.bool(),
            rng.range(Decimal::MAX_SCALE as u64 + 1, 255) as u32,
        );
        for (x, y) in [(a, b), (b, a)] {
            if let CalculationResult::Ok(d) = mul_impl(&x, &y) {
                assert!(d.scale() <= Decimal::MAX_SCALE && d.mantissa().unsigned_abs() <= MANTISSA_MAX);
            }
        }
    }
}

#[test]
fn decimal_digits_matches_string_length() {
    let mut rng = Rng(0x5851_F42D_4C95_7F2D);
    let mut values = vec![1u128, 9, 10, 11, u32::MAX as u128, u64::MAX as u128, MANTISSA_MAX];
    for k in 1..=38 {
        values.extend([pow10(k) - 1, pow10(k), pow10(k) + 1]);
    }
    for _ in 0..100_000 {
        values.push(rng.bits_in(1, 127));
    }
    for v in values {
        let expected = v.to_string().len() as u32;
        assert_eq!(decimal_digits_u128(v), expected, "{v}");
        if v <= u32::MAX as u128 {
            assert_eq!(decimal_digits_u32(v as u32), expected, "{v}");
        }
    }
}

/// Multiplies two 128 bit values, returning the little endian 256 bit product.
fn mul_wide(a: u128, b: u128) -> [u64; 4] {
    let (a0, a1) = (a as u64 as u128, a >> 64);
    let (b0, b1) = (b as u64 as u128, b >> 64);
    let low = a0 * b0;
    let (cross, carry) = (a0 * b1).overflowing_add(a1 * b0);
    let mid = (low >> 64) + (cross as u64 as u128);
    let high = a1 * b1 + (cross >> 64) + ((carry as u128) << 64) + (mid >> 64);
    [low as u64, mid as u64, high as u64, (high >> 64) as u64]
}

/// Returns `q * d + r` as a little endian 192 bit value, or `None` if it doesn't fit.
fn compose(q: u128, d: u128, r: u128) -> Option<[u64; 3]> {
    let [x0, x1, x2, x3] = mul_wide(q, d);
    let (low, carry) = (((x1 as u128) << 64) | x0 as u128).overflowing_add(r);
    let (x2, carry) = x2.overflowing_add(carry as u64);
    if x3 != 0 || carry {
        return None;
    }
    Some([low as u64, (low >> 64) as u64, x2])
}

/// Quotients up to 128 bits and remainders (biased towards the rounding boundaries) for `d`.
fn quotient_and_remainder_cases(rng: &mut Rng, d: u128) -> Vec<(u128, u128)> {
    let mut cases = vec![];
    for _ in 0..20_000 {
        let q = match rng.below(4) {
            0 => u128::MAX >> rng.below(128),
            _ => rng.bits_in(0, 128),
        };
        let r = match rng.below(5) {
            0 => 0,
            1 => d - 1,
            2 => d / 2,
            3 => d / 2 + 1,
            _ => rng.u128() % d,
        };
        cases.push((q, r));
    }
    cases
}

#[test]
fn pow10_divisors_divide_correctly() {
    let mut rng = Rng(0x1405_7B7E_F767_814F);
    for (power, divisor) in POW10_DIVISORS.iter().enumerate().skip(1) {
        let d = pow10(power as u32);
        assert_eq!(divisor.divisor as u128, d << divisor.shift);
        assert_eq!(divisor.divisor >> 63, 1);
        assert_eq!(
            divisor.reciprocal as u128,
            u128::MAX / divisor.divisor as u128 - (1 << 64)
        );
        for (q, r) in quotient_and_remainder_cases(&mut rng, d) {
            let Some(x) = compose(q, d, r) else { continue };
            let (quotient, remainder) = divisor.div_rem_to_128(x);
            assert_eq!(
                (quotient, remainder >> divisor.shift),
                (q, r as u64),
                "{x:?} / 10^{power}"
            );
            assert_eq!(remainder & ((1 << divisor.shift) - 1), 0);
        }
    }
}

#[test]
fn pow10_divisors_128_divide_correctly() {
    let mut rng = Rng(0x9E6C_63D0_676A_9A99);
    for (i, divisor) in POW10_DIVISORS_128.iter().enumerate() {
        let power = (MAX_POW10_U64 + 1 + i) as u32;
        let d = pow10(power);
        assert_eq!(divisor.divisor, d << divisor.shift);
        assert_eq!(divisor.divisor >> 127, 1);
        for (q, r) in quotient_and_remainder_cases(&mut rng, d) {
            let Some(x) = compose(q, d, r) else { continue };
            let (quotient, remainder) = divisor.div_rem_to_128(x);
            assert_eq!((quotient, remainder >> divisor.shift), (q, r), "{x:?} / 10^{power}");
        }
    }
}
