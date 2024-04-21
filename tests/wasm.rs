use rust_decimal::Decimal;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn convert() {
    let d = Decimal::from_number(42.0);
    assert_eq!(d.unwrap().to_number(), 42.0);
}
