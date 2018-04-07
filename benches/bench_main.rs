#[macro_use]
extern crate criterion;
#[cfg(feature = "comparitive")]
extern crate decimal;
#[macro_use]
extern crate lazy_static;
extern crate rust_decimal;

mod std_ops;
#[cfg(feature = "comparitive")]
mod comparitive;
#[cfg(not(feature = "comparitive"))]
mod comparitive {
    pub fn benches() {}
}

criterion_main!{
    std_ops::benches,
    comparitive::benches
}
