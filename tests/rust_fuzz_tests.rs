use arbitrary::{Arbitrary, Unstructured};
use rust_decimal::Decimal;

#[test]
fn it_can_generate_arbitrary_decimals() {
    let mut u = Unstructured::new(b"it_can_generate_arbitrary_decimals");
    let d = Decimal::arbitrary(&mut u);
    assert!(d.is_ok());
}
