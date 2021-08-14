use core::str::FromStr;
use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};

macro_rules! add_benchmark_group {
    ($criterion:expr, $f:ident, $op:tt) => {
        fn $f<M, const N: usize>(group: &mut BenchmarkGroup<'_, M>)
        where
        M: Measurement,
        {
            group.bench_with_input(BenchmarkId::new("bigdecimal", N), &N, |ben, _| {
                let a = bigdecimal::BigDecimal::from_str("2.01").unwrap();
                let b = bigdecimal::BigDecimal::from_str("2.01").unwrap();
                ben.iter(|| black_box(a.clone() $op b.clone()))
            });

            group.bench_with_input(BenchmarkId::new("decimal-rs", N), &N, |ben, _| {
                let a = decimal_rs::d128!(2.01);
                let b = decimal_rs::d128!(2.01);
                ben.iter(|| black_box(a $op b))
            });

            group.bench_with_input(BenchmarkId::new("f32", N), &N, |ben, _| {
                let a = 2.01f32;
                let b = 2.01f32;
                ben.iter(|| black_box(a $op b))
            });

            group.bench_with_input(BenchmarkId::new("f64", N), &N, |ben, _| {
                let a = 2.01f64;
                let b = 2.01f64;
                ben.iter(|| black_box(a $op b))
            });

            group.bench_with_input(BenchmarkId::new("rust-decimal", N), &N, |ben, _| {
                let a = rust_decimal_macros::dec!(2.01);
                let b = rust_decimal_macros::dec!(2.01);
                ben.iter(|| black_box(a $op b))
            });
        }

        let mut group = $criterion.benchmark_group(stringify!($f));
        $f::<_, 100>(&mut group);
        group.finish();
    };
}

fn criterion_benchmark(c: &mut Criterion) {
    add_benchmark_group!(c, addition, +);
    add_benchmark_group!(c, division, /);
    add_benchmark_group!(c, multiplication, *);
    add_benchmark_group!(c, subtraction, -);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
