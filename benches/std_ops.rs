use criterion::Criterion;
use rust_decimal::Decimal;
use std::str::FromStr;

lazy_static! {
    static ref BASE : Decimal = Decimal::from_str("2.01").unwrap();
    static ref INPUTS : [Decimal; 7] = [
        Decimal::from_str("1").unwrap(),
        Decimal::from_str("2").unwrap(),
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("0.01").unwrap(),
        Decimal::from_str("-0.5").unwrap(),
        Decimal::from_str("3.1415926535897932384626433832").unwrap(),
        Decimal::from_str("-3.1415926535897932384626433832").unwrap(),
    ];
}

macro_rules! bench_decimal_op {
    ($name:ident, $op:tt) => {
        fn $name(c: &mut Criterion) {
            c.bench_function_over_inputs(stringify!($name),
                |b, &y| b.iter(|| {
                    let _ = *BASE $op *y;
                }),
                INPUTS.iter(),
            );
        }
    }
}

bench_decimal_op!(add, +);
bench_decimal_op!(sub, -);
bench_decimal_op!(mul, *);
bench_decimal_op!(div, /);

criterion_group!(benches,
    add,
    sub,
    mul,
    div
);
