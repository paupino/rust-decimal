#![feature(test)]

extern crate test;
extern crate rust_decimal;
extern crate decimal;
use decimal::d128;
use rust_decimal::Decimal;
use std::str::FromStr;
use test::Bencher;

macro_rules! bench_bin_op {
    ($name:ident, $ty:ident, $op:tt) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {

                let y = $ty::from_str("2").unwrap();
                for _ in 0..100 {
                    let mut x = y;
                    for _ in 0..50 {
                        x = x $op y;
                    }
                    test::black_box(x);
                }
            });
        }

    }
}

bench_bin_op!(bench_decimal_add, Decimal, +);
bench_bin_op!(bench_d128_add, d128, +);


bench_bin_op!(bench_decimal_sub, Decimal, -);
bench_bin_op!(bench_d128_sub, d128, -);


bench_bin_op!(bench_decimal_mul, Decimal, *);
bench_bin_op!(bench_d128_mul, d128, *);

bench_bin_op!(bench_decimal_div, Decimal, /);
bench_bin_op!(bench_d128_div, d128, /);
