//! Micro-benchmarks targeting the operations affected by the performance work.
//! Inputs are passed through `black_box` so the compiler cannot const-fold them.

use core::hash::{Hash, Hasher};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_decimal::Decimal;

// 64-bit mantissas (hi == 0, mid != 0) to exercise the 64-bit fast paths.
const A_64: Decimal = Decimal::from_parts(0x1234_5678, 0x9ABC, 0, false, 6);
const B_64: Decimal = Decimal::from_parts(0x90AB_CDEF, 0x1357, 0, false, 6);
// Same magnitudes but different scales (exercises rescale alignment).
const A_64_S2: Decimal = Decimal::from_parts(0x1234_5678, 0x9ABC, 0, false, 2);
const B_64_S8: Decimal = Decimal::from_parts(0x90AB_CDEF, 0x1357, 0, false, 8);
// 96-bit mantissa.
const C_96: Decimal = Decimal::from_parts(0x1234_5678, 0x9ABC_DEF0, 0x1357, false, 10);
// Same mantissa/scale as C_96 but opposite sign: identical lo/mid/hi, only flags differ. This is
// the worst case for the equality fast path (all four field compares run, then it still falls
// back to the full comparison).
const C_96_NEG: Decimal = Decimal::from_parts(0x1234_5678, 0x9ABC_DEF0, 0x1357, true, 10);
// Numerically equal values with different scales (1.5 == 1.50): different bit patterns, so the
// fast path misses and the comparison runs.
const EQ_15: Decimal = Decimal::from_parts(15, 0, 0, false, 1);
const EQ_150: Decimal = Decimal::from_parts(150, 0, 0, false, 2);
// 64-bit operands whose aligned sum exceeds 96 bits (10_000_000_000 == 0x2_540B_E400 at scale 0,
// plus a value at scale 19): the add fast path computes the u128 result, sees it overflow 96 bits
// and discards it, falling back to the slow path.
const ADD_BIG: Decimal = Decimal::from_parts(0x540B_E400, 0x2, 0, false, 0);
const ADD_SMALL_S19: Decimal = Decimal::from_parts(1, 0, 0, false, 19);
// Even mantissa with no trailing zeros (0.00012): normalize finds nothing to strip but still has
// to check.
const NORM_NOSTRIP: Decimal = Decimal::from_parts(12, 0, 0, false, 5);
// Odd mantissa (19.99): cannot have trailing zeros, exercises the odd fast path.
const NORM_ODD: Decimal = Decimal::from_parts(1999, 0, 0, false, 2);

struct NopHasher(u64);
impl Hasher for NopHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*b as u64);
        }
    }
}

fn benches(c: &mut Criterion) {
    c.bench_function("mul_64", |b| b.iter(|| black_box(A_64) * black_box(B_64)));
    c.bench_function("mul_96", |b| b.iter(|| black_box(C_96) * black_box(B_64)));

    c.bench_function("add_64_same_scale", |b| b.iter(|| black_box(A_64) + black_box(B_64)));
    c.bench_function("add_64_diff_scale", |b| {
        b.iter(|| black_box(A_64_S2) + black_box(B_64_S8))
    });
    c.bench_function("sub_64_diff_scale", |b| {
        b.iter(|| black_box(A_64_S2) - black_box(B_64_S8))
    });

    c.bench_function("rescale_up", |b| b.iter(|| black_box(A_64).rescale(black_box(20))));
    c.bench_function("rescale_down", |b| b.iter(|| black_box(C_96).rescale(black_box(2))));
    c.bench_function("round_dp", |b| b.iter(|| black_box(C_96).round_dp(black_box(3))));
    c.bench_function("trunc", |b| b.iter(|| black_box(C_96).trunc()));

    // Trailing-zero heavy value for normalize/hash.
    let trailing = Decimal::from_parts(1_000_000, 0, 0, false, 10);
    c.bench_function("normalize", |b| b.iter(|| black_box(trailing).normalize()));
    c.bench_function("hash", |b| {
        b.iter(|| {
            let mut h = NopHasher(0);
            black_box(trailing).hash(&mut h);
            h.finish()
        })
    });

    c.bench_function("floor", |b| b.iter(|| black_box(C_96).floor()));
    c.bench_function("ceil", |b| b.iter(|| black_box(C_96).ceil()));

    c.bench_function("eq_equal", |b| b.iter(|| black_box(C_96) == black_box(C_96)));
    // Differs in the first limb -> fast path rejects on the first compare, then falls to cmp.
    c.bench_function("eq_neq_lo", |b| b.iter(|| black_box(A_64) == black_box(B_64)));
    // Differs only in flags -> all four field compares run before falling to cmp (worst case).
    c.bench_function("eq_neq_flags", |b| b.iter(|| black_box(C_96) == black_box(C_96_NEG)));
    // Equal value, different scale -> fast path misses, cmp returns equal.
    c.bench_function("eq_equal_diff_scale", |b| {
        b.iter(|| black_box(EQ_15) == black_box(EQ_150))
    });

    // Add where the 64-bit fast path computes a result that overflows 96 bits and is discarded.
    c.bench_function("add_64_overflow", |b| {
        b.iter(|| black_box(ADD_BIG) + black_box(ADD_SMALL_S19))
    });
    // Normalize on an even mantissa that has no trailing zeros to strip.
    c.bench_function("normalize_nostrip", |b| b.iter(|| black_box(NORM_NOSTRIP).normalize()));
    // Normalize on an odd mantissa (hits the odd fast path).
    c.bench_function("normalize_odd", |b| b.iter(|| black_box(NORM_ODD).normalize()));
}

criterion_group!(g, benches);
criterion_main!(g);
