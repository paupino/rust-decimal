use criterion::{Criterion, Fun};
use decimal::d128;
use rust_decimal::Decimal;
use std::str::FromStr;

lazy_static! {
    static ref BASE_DECIMAL : Decimal = Decimal::from_str("2.01").unwrap();
    static ref BASE_D128 : d128 = d128::from_str("2.01").unwrap();
    static ref COMPARE_DECIMAL : Decimal = Decimal::from_str("3.1415926535897932384626433832").unwrap();
    static ref COMPARE_D128 : d128 = d128::from_str("3.1415926535897932384626433832").unwrap();
}

macro_rules! bench_compare_op {
    ($name:ident, $op:tt) => {
        fn $name(c: &mut Criterion) {
            let impl_rust = Fun::new("Decimal", |b, _| b.iter(|| {
                let _ = *BASE_DECIMAL $op *COMPARE_DECIMAL;
            }));
            let impl_c = Fun::new("d128", |b, _| b.iter(|| {
                let _ = *BASE_D128 $op *COMPARE_D128;
            }));

            let functions = vec!(impl_rust, impl_c);
            c.bench_functions(stringify!($name), functions, &0);
        }
    }
}

bench_compare_op!(add, +);
bench_compare_op!(sub, -);
bench_compare_op!(mul, *);
bench_compare_op!(div, /);

criterion_group!(benches,
    add,
    sub,
    mul,
    div
);

