#![no_main]

use rust_decimal::{Decimal, MathematicalOps};

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    a: Decimal,
    b: Decimal,
    exp_f64: f64,
    exp_i64: i64,
    exp_u64: u64,
}

libfuzzer_sys::fuzz_target!(|data: Data| {
    let fun = || {
        let _ = data.a.checked_add(data.b)?;
        let _ = data.a.checked_div(data.b)?;
        //let _ = data.a.checked_exp_with_tolerance(data.b)?;
        //let _ = data.a.checked_exp()?;
        //let _ = data.a.checked_mul(data.b)?;
        //let _ = data.a.checked_norm_pdf()?;
        //let _ = data.a.checked_powd(data.b)?;
        //let _ = data.a.checked_powf(data.exp_f64)?;
        //let _ = data.a.checked_powi(data.exp_i64)?;
        //let _ = data.a.checked_powu(data.exp_u64)?;
        //let _ = data.a.checked_sub(data.b)?;

        Some(())
    };
    let _ = fun();
});
