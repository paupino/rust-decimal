extern crate rust_decimal;
extern crate rust_decimal_macro;

use rust_decimal_macro::*;

#[test]
fn it_can_parse_decimal() {
    let d = d!(123.45);
    assert_eq!("123.45", d.to_string());
}