#![feature(test)]

extern crate rust_decimal;
extern crate test;

use rust_decimal::Decimal;
use std::str::FromStr;

macro_rules! bench_decimal_op {
    ($name:ident, $op:tt, $y:expr) => {
        #[bench]
        fn $name(b: &mut ::test::Bencher) {
            let x = Decimal::from_str("2.01").unwrap();
            let y = Decimal::from_str($y).unwrap();
            b.iter(|| {
                let result = x $op y;
                ::test::black_box(result);
            });
        }
    }
}

macro_rules! bench_fold_op {
    ($name:ident, $op:tt, $init:expr, $count:expr) => {
        #[bench]
        fn $name(b: &mut ::test::Bencher) {
            fn fold(values: &[Decimal]) -> Decimal {
                let mut acc: Decimal = $init.into();
                for value in values {
                    acc = acc $op value;
                }
                acc
            }

            let values: Vec<Decimal> = test::black_box((1..$count).map(|i| i.into()).collect());
            b.iter(|| {
                let result = fold(&values);
                ::test::black_box(result);
            });
        }
    }
}

/* Add */
bench_decimal_op!(add_one, +, "1");
bench_decimal_op!(add_two, +, "2");
bench_decimal_op!(add_one_hundred, +, "100");
bench_decimal_op!(add_point_zero_one, +, "0.01");
bench_decimal_op!(add_negative_point_five, +, "-0.5");
bench_decimal_op!(add_pi, +, "3.1415926535897932384626433832");
bench_decimal_op!(add_negative_pi, +, "-3.1415926535897932384626433832");

bench_fold_op!(add_10k, +, 0, 10_000);

/* Sub */
bench_decimal_op!(sub_one, -, "1");
bench_decimal_op!(sub_two, -, "2");
bench_decimal_op!(sub_one_hundred, -, "100");
bench_decimal_op!(sub_point_zero_one, -, "0.01");
bench_decimal_op!(sub_negative_point_five, -, "-0.5");
bench_decimal_op!(sub_pi, -, "3.1415926535897932384626433832");
bench_decimal_op!(sub_negative_pi, -, "-3.1415926535897932384626433832");

bench_fold_op!(sub_10k, -, 5_000_000, 10_000);

/* Mul */
bench_decimal_op!(mul_one, *, "1");
bench_decimal_op!(mul_two, *, "2");
bench_decimal_op!(mul_one_hundred, *, "100");
bench_decimal_op!(mul_point_zero_one, *, "0.01");
bench_decimal_op!(mul_negative_point_five, *, "-0.5");
bench_decimal_op!(mul_pi, *, "3.1415926535897932384626433832");
bench_decimal_op!(mul_negative_pi, *, "-3.1415926535897932384626433832");

/* Div */
bench_decimal_op!(div_one, /, "1");
bench_decimal_op!(div_two, /, "2");
bench_decimal_op!(div_one_hundred, /, "100");
bench_decimal_op!(div_point_zero_one, /, "0.01");
bench_decimal_op!(div_negative_point_five, /, "-0.5");
bench_decimal_op!(div_pi, /, "3.1415926535897932384626433832");
bench_decimal_op!(div_negative_pi, /, "-3.1415926535897932384626433832");

bench_fold_op!(div_10k, /, Decimal::max_value(), 10_000);

#[cfg(feature = "postgres")]
mod postgres {
    #[cfg(feature = "postgres")]
    extern crate postgres as pg_crate;

    use super::*;
    use postgres::pg_crate::types::{Kind, Type, ToSql, FromSql};

    #[bench]
    fn to_from_sql(b: &mut ::test::Bencher) {
        let samples_strs = &[
            "3950.123456",
            "3950",
            "0.1",
            "0.01",
            "0.001",
            "0.0001",
            "0.00001",
            "0.000001",
            "1",
            "-100",
            "-123.456",
            "119996.25",
            "1000000",
            "9999999.99999",
            "12340.56789"
        ];

        let samples: Vec<Decimal> = test::black_box(samples_strs.iter().map(|x| Decimal::from_str(x).unwrap()).collect());
        let t = Type::_new("".into(), 0, Kind::Simple, "".into());
        let mut vec = Vec::<u8>::with_capacity(100);

        b.iter(|| {
            for _ in 0..100 {
                for sample in &samples {
                    vec.clear();
                    sample.to_sql(&t, &mut vec).unwrap();
                    let result = Decimal::from_sql(&t, &vec).unwrap();
                    ::test::black_box(result);
                }
            }
        });
    }
}
