// Tests for `mul_impl`. `reference` computes the exact product and rounds it one digit at a time,
// and `mul_impl` is compared with it bit for bit over a deterministic sample of inputs biased
// towards edge cases.

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

fn word_edge(rng: &mut Rng) -> u32 {
    match rng.below(8) {
        0 => 0,
        1 => 1,
        2 => u32::MAX,
        3 => u32::MAX - 1,
        4 => 0x8000_0000,
        5 => 0x7FFF_FFFF,
        _ => rng.next() as u32,
    }
}

fn special_mantissa(rng: &mut Rng) -> u128 {
    let k = rng.below(29) as u32;
    match rng.below(16) {
        0 => rng.range(0, 10) as u128,
        1 => u32::MAX as u128,
        2 => 1u128 << 32,
        3 => u64::MAX as u128,
        4 => 1u128 << 64,
        5 => MANTISSA_MAX,
        6 => MANTISSA_MAX - rng.below(16) as u128,
        7 => 1u128 << rng.below(96),
        8 => (1u128 << rng.range(1, 96)) - 1,
        9 => pow10(k),
        10 => pow10(k) - 1,
        11 => pow10(k) + 1,
        12 => (5 * pow10(k)).min(MANTISSA_MAX),
        13 => (5 * pow10(k)).min(MANTISSA_MAX) - 1,
        14 => (5 * pow10(k)).min(MANTISSA_MAX - 1) + 1,
        _ => (1u128 << 32) + rng.below(3) as u128 - 1,
    }
}

/// A mantissa biased towards the shapes that exercise different code paths: word boundaries,
/// specific widths, decimal-looking values with trailing zeros and 5s (rounding ties).
fn mantissa(rng: &mut Rng) -> u128 {
    match rng.below(10) {
        0 | 1 => special_mantissa(rng),
        2 | 3 => rng.bits_in(1, 96),
        4 => {
            let lo = word_edge(rng) as u128;
            let mid = word_edge(rng) as u128;
            let hi = word_edge(rng) as u128;
            match rng.below(3) {
                0 => lo,
                1 => lo | (mid << 32),
                _ => lo | (mid << 32) | (hi << 64),
            }
        }
        5 | 6 => {
            // Small digits followed by zeros, optionally with a trailing 5 or +-1 for rounding ties.
            let digits = rng.bits_in(1, 40);
            let zeros = rng.below(29) as u32;
            let base = digits.saturating_mul(pow10(zeros));
            let v = match rng.below(4) {
                0 if zeros > 0 => base.saturating_add(5 * pow10(zeros - 1)),
                1 => base.saturating_add(1),
                2 => base.saturating_sub(1),
                _ => base,
            };
            v.min(MANTISSA_MAX)
        }
        _ => rng.u128() & MANTISSA_MAX,
    }
}

fn decimal(mantissa: u128, negative: bool, scale: u32) -> Decimal {
    assert!(mantissa <= MANTISSA_MAX && scale <= Decimal::MAX_SCALE);
    // Deliberately allow negative zero: it's representable and can come out of multiplication.
    Decimal::from_parts_raw_unchecked(
        mantissa as u32,
        (mantissa >> 32) as u32,
        (mantissa >> 64) as u32,
        crate::decimal::flags(negative, scale),
    )
}

fn scale(rng: &mut Rng) -> u32 {
    match rng.below(4) {
        0 => rng.range(20, 28) as u32,
        1 => rng.range(0, 4) as u32,
        _ => rng.below(29) as u32,
    }
}

fn random_decimal(rng: &mut Rng) -> Decimal {
    let m = mantissa(rng);
    let negative = rng.bool();
    let s = scale(rng);
    decimal(m, negative, s)
}

fn describe(r: &CalculationResult) -> Option<(u32, u32, u32, u32)> {
    match r {
        CalculationResult::Ok(d) => Some((d.lo(), d.mid(), d.hi(), d.flags())),
        CalculationResult::Overflow => None,
        CalculationResult::DivByZero => panic!("mul returned DivByZero"),
    }
}

/// Divides the little endian 256 bit `x` by 10 in place, returning the remainder.
fn div10(x: &mut [u64; 4]) -> u64 {
    let mut rem = 0u128;
    for word in x.iter_mut().rev() {
        let cur = (rem << 64) | *word as u128;
        *word = (cur / 10) as u64;
        rem = cur % 10;
    }
    rem as u64
}

/// The expected result of `a * b`: the exact product rounded half to even, dropping the fewest
/// digits that make the mantissa fit in 96 bits and the scale at most 28. Returns `None` on
/// overflow.
fn reference(a: &Decimal, b: &Decimal) -> Option<(u32, u32, u32, u32)> {
    if a.is_zero() || b.is_zero() {
        return Some((0, 0, 0, 0));
    }
    let (ma, mb) = (a.mantissa().unsigned_abs(), b.mantissa().unsigned_abs());
    let negative = a.is_sign_negative() ^ b.is_sign_negative();
    let scale = a.scale() + b.scale();
    // 32x32 bit products with a scale above 47 are returned as zero without rounding.
    let small = ma >> 32 == 0 && mb >> 32 == 0;
    if small && scale > Decimal::MAX_SCALE + MAX_I64_SCALE {
        return Some((0, 0, 0, 0));
    }
    let product = mul_wide(ma, mb);
    for k in scale.saturating_sub(Decimal::MAX_SCALE)..=scale {
        // Drop k digits, keeping the last one dropped and whether any before it were non-zero.
        let mut q = product;
        let (mut last, mut sticky) = (0, 0);
        for _ in 0..k {
            sticky |= last;
            last = div10(&mut q);
        }
        let low = ((q[1] as u128) << 64) | q[0] as u128;
        let round_up = last > 5 || (last == 5 && (sticky != 0 || low & 1 == 1));
        // Can't overflow: once a digit has been dropped, the quotient is below 2^189.
        let m = low + round_up as u128;
        if q[2] != 0 || q[3] != 0 || m > MANTISSA_MAX {
            continue;
        }
        // 32x32 bit products that round to zero lose their sign; wider ones keep it (#842).
        let d = decimal(m, negative && !(m == 0 && small), scale - k);
        return Some((d.lo(), d.mid(), d.hi(), d.flags()));
    }
    None
}

fn check(a: &Decimal, b: &Decimal) {
    let expected = reference(a, b);
    let actual = describe(&mul_impl(a, b));
    assert_eq!(
        expected,
        actual,
        "mul mismatch for {:?} ({:?}) * {:?} ({:?})",
        a,
        (a.lo(), a.mid(), a.hi(), a.flags()),
        b,
        (b.lo(), b.mid(), b.hi(), b.flags())
    );
}

fn check_both_orders(a: &Decimal, b: &Decimal) {
    check(a, b);
    check(b, a);
}

#[test]
fn mul_matches_reference_random() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..150_000 {
        let a = random_decimal(&mut rng);
        let b = random_decimal(&mut rng);
        check(&a, &b);
    }
}

#[test]
fn mul_matches_reference_all_scales() {
    // Every scale combination for a spread of mantissa pairs.
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..150 {
        let m1 = mantissa(&mut rng);
        let m2 = mantissa(&mut rng);
        let negative = rng.bool();
        for s1 in 0..=Decimal::MAX_SCALE {
            for s2 in 0..=Decimal::MAX_SCALE {
                check(&decimal(m1, negative, s1), &decimal(m2, !negative, s2));
            }
        }
    }
}

#[test]
fn mul_matches_reference_product_near_96_bits() {
    // Products just above and below 2^96, without and with scale driven rounding.
    let mut rng = Rng(0x94D0_49BB_1331_11EB);
    for _ in 0..50_000 {
        let a = rng.bits_in(1, 96);
        let target = (1u128 << 96)
            .wrapping_add(rng.below(1 << 20) as u128)
            .wrapping_sub(1 << 19);
        let b = target / a;
        if b == 0 || b > MANTISSA_MAX {
            continue;
        }
        for b in [b.saturating_sub(1), b, b + 1] {
            if b == 0 || b > MANTISSA_MAX {
                continue;
            }
            let da = decimal(a, rng.bool(), scale(&mut rng));
            let db = decimal(b, rng.bool(), scale(&mut rng));
            check_both_orders(&da, &db);
        }
    }
}

#[test]
fn mul_matches_reference_rounding_near_96_bit_boundary() {
    // Products a little above (2^96 - 1) * 10^k, so that dropping k digits often leaves a quotient
    // of 2^96 - 1: when that rounds up, the carry out of 96 bits forces another digit to be dropped.
    // Exact ties on that quotient are covered below.
    let mut rng = Rng(0xBF58_476D_1CE4_E5B9);
    for _ in 0..50_000 {
        let k = rng.range(1, 9) as u32;
        let p = pow10(k);
        let target = MANTISSA_MAX * p;
        let a = p + rng.u128() % (3 * p);
        let b = target.div_ceil(a);
        if b > MANTISSA_MAX {
            continue;
        }
        let s1 = rng.below(29) as u32;
        let s2 = rng.below(29) as u32;
        check_both_orders(&decimal(a, rng.bool(), s1), &decimal(b, rng.bool(), s2));
    }

    // (2^97 - 1) * 5 * 10^(k - 1) == (2^96 - 1) * 10^k + 5 * 10^(k - 1): an exact tie on an odd
    // quotient of 2^96 - 1, so rounding carries out of 96 bits. 2^97 - 1 = 11447 * 13842607235828485645766393.
    const F1: u128 = 11447;
    const F2: u128 = 13842607235828485645766393;
    assert_eq!(F1 * F2, (1u128 << 97) - 1);
    for k in 1..=24u32 {
        let a = 5 * F1 * pow10(k - 1);
        if a > MANTISSA_MAX {
            continue;
        }
        for da in [a - 1, a, a + 1] {
            for db in [F2 - 1, F2, F2 + 1] {
                for s1 in 0..=Decimal::MAX_SCALE {
                    for s2 in 0..=Decimal::MAX_SCALE {
                        check_both_orders(&decimal(da, false, s1), &decimal(db, true, s2));
                    }
                }
            }
        }
    }
}

#[test]
fn mul_matches_reference_rounding_ties() {
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..50_000 {
        // Bit length driven: x = 5 * 10^(k-1) * (2q + 1) == q * 10^k + 5 * 10^(k-1), an exact tie
        // when k digits are dropped. q in [2^96 / 10, 2^95) so exactly k digits need dropping.
        let k = rng.range(1, 27) as u32;
        let half = 5 * pow10(k - 1);
        let q = (1u128 << 96) / 10 + rng.u128() % ((1u128 << 95) - (1u128 << 96) / 10);
        let b = 2 * q + 1;
        let a = match rng.below(4) {
            0 => half - 1,
            1 => half + 1,
            _ => half,
        };
        if a <= MANTISSA_MAX {
            let s1 = rng.below(29) as u32;
            let s2 = rng.below(29) as u32;
            check_both_orders(&decimal(a, rng.bool(), s1), &decimal(b, rng.bool(), s2));
        }

        // Scale driven: the product usually fits in 96 bits but the combined scale exceeds 28 by `drop`, and
        // the dropped digits are exactly 5 * 10^(drop-1) (optionally perturbed to set sticky bits).
        let s1 = rng.range(1, 28) as u32;
        let s2 = rng.range(29 - s1 as u64, 28) as u32;
        let drop = s1 + s2 - Decimal::MAX_SCALE;
        let odd = rng.bits_in(1, 30) | 1;
        let a = match rng.below(4) {
            0 => 5 * pow10(drop - 1) * odd + 1,
            1 => (5 * pow10(drop - 1) * odd).saturating_sub(1),
            _ => 5 * pow10(drop - 1) * odd,
        };
        let b = rng.bits_in(1, 30) | 1;
        if a <= MANTISSA_MAX && a > 0 {
            check_both_orders(&decimal(a, rng.bool(), s1), &decimal(b, rng.bool(), s2));
        }
    }
}

#[test]
fn mul_matches_reference_underflow_and_trivial() {
    let mut rng = Rng(0x0123_4567_89AB_CDEF);
    for _ in 0..30_000 {
        // Tiny values at high scales round towards (possibly negative) zero.
        let a = decimal(rng.bits_in(1, 64), rng.bool(), rng.range(14, 28) as u32);
        let b = decimal(rng.bits_in(1, 64), rng.bool(), rng.range(14, 28) as u32);
        check(&a, &b);

        // Multiplying by zero (both signs, any scale), one and minus one.
        let x = random_decimal(&mut rng);
        let s = rng.below(29) as u32;
        let negative = rng.bool();
        for trivial in [
            decimal(0, negative, s),
            decimal(1, negative, s),
            decimal(pow10(s), negative, s),
        ] {
            check_both_orders(&x, &trivial);
        }
    }
}

#[test]
fn mul_matches_reference_extremes() {
    let edges: [u128; 12] = [
        1,
        2,
        5,
        9,
        10,
        u32::MAX as u128,
        1u128 << 32,
        u64::MAX as u128,
        1u128 << 64,
        MANTISSA_MAX / 10,
        MANTISSA_MAX - 1,
        MANTISSA_MAX,
    ];
    for &m1 in &edges {
        for &m2 in &edges {
            for s1 in 0..=Decimal::MAX_SCALE {
                for s2 in 0..=Decimal::MAX_SCALE {
                    check(&decimal(m1, false, s1), &decimal(m2, true, s2));
                }
            }
        }
    }
}

#[test]
fn mul_matches_reference_scale_above_max() {
    // Operands with a scale above 28 can't come from the checked constructors but can be built
    // through `bytemuck`; multiplying them must not panic.
    let mut rng = Rng(0x6A09_E667_F3BC_C909);
    for _ in 0..10_000 {
        let m1 = mantissa(&mut rng);
        let m2 = mantissa(&mut rng);
        let s1 = rng.range(0, 255) as u32;
        let s2 = rng.range(Decimal::MAX_SCALE as u64 + 1, 255) as u32;
        let raw = |m: u128, negative: bool, scale: u32| {
            Decimal::from_parts_raw_unchecked(
                m as u32,
                (m >> 32) as u32,
                (m >> 64) as u32,
                crate::decimal::flags(negative, scale),
            )
        };
        check_both_orders(&raw(m1, rng.bool(), s1), &raw(m2, rng.bool(), s2));
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
