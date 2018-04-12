#![feature(test)]
#![feature(plugin)]
#![plugin(interpolate_idents)]

#[cfg(feature = "comparitive")]
extern crate decimal;
extern crate rust_decimal;
extern crate test;

use rust_decimal::Decimal;
use std::str::FromStr;

macro_rules! bench_decimal_op {
    ($name:ident, $op:tt) => {
        
        bench_decimal_op_value!($name, one, $op, "1");
        bench_decimal_op_value!($name, two, $op, "2");
        bench_decimal_op_value!($name, one_hundred, $op, "100");
        bench_decimal_op_value!($name, point_zero_one, $op, "0.01");
        bench_decimal_op_value!($name, negative_point_five, $op, "-0.5");
        bench_decimal_op_value!($name, pi, $op, "3.1415926535897932384626433832");
        bench_decimal_op_value!($name, negative_pi, $op, "-3.1415926535897932384626433832");
    }
}

macro_rules! bench_decimal_op_value {
    ($name:ident, $value_str:ident, $op:tt, $y:expr) => {
        interpolate_idents! {
            #[bench]
            fn [$name _ $value_str](b: &mut ::test::Bencher) {
                let x = Decimal::from_str("2.01").unwrap();
                let y = Decimal::from_str($y).unwrap();
                b.iter(|| {
                    let result = x $op y;
                    ::test::black_box(result);
                });
            }
        }
    }
}

bench_decimal_op!(add, +);
bench_decimal_op!(sub, -);
bench_decimal_op!(mul, *);
bench_decimal_op!(div, /);

#[cfg(feature = "comparitive")]
mod comparitive {
    use decimal::d128;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    macro_rules! bench_compare_op {
        ($name:ident, $op:tt) => {
            interpolate_idents! {
                #[bench]
                fn [$name _dec](b: &mut ::test::Bencher) {
                    let x = Decimal::from_str("2.01").unwrap();
                    let y = Decimal::from_str("3.1415926535897932384626433832").unwrap();
                    b.iter(|| {
                        let result = x $op y;
                        ::test::black_box(result);
                    });
                }
            }

            interpolate_idents! {
                #[bench]
                fn [$name _d128](b: &mut ::test::Bencher) {
                    let x = d128::from_str("2.01").unwrap();
                    let y = d128::from_str("3.1415926535897932384626433832").unwrap();
                    b.iter(|| {
                        let result = x $op y;
                        ::test::black_box(result);
                    });
                }
            }
        }
    }

    bench_compare_op!(add, +);
    bench_compare_op!(sub, -);
    bench_compare_op!(mul, *);
    bench_compare_op!(div, /);
}
#[cfg(not(feature = "comparitive"))]
mod comparitive {
    
}