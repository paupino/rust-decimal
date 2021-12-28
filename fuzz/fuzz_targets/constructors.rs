#![no_main]

use rust_decimal::Decimal;

#[derive(Debug, arbitrary::Arbitrary)]
struct Data<'a> {
    generic_str: &'a str,

    try_from_i128_with_scale_num: i128,
    try_from_i128_with_scale_scale: u32,

    try_new_num: i64,
    try_new_scale: u32,
}

libfuzzer_sys::fuzz_target!(|data: Data<'_>| {
    let _ = serde_json::from_str::<Decimal>(data.generic_str);

    let _ = Decimal::from_scientific(data.generic_str);

    let _ = Decimal::try_from_i128_with_scale(data.try_from_i128_with_scale_num, data.try_from_i128_with_scale_scale);

    let _ = Decimal::try_new(data.try_new_num, data.try_new_scale);
});
