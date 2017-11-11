extern crate rust_decimal;
extern crate rust_decimal_macro;

use rust_decimal_macro::*;

#[test]
fn it_can_parse_decimal() {
    assert_eq!("0.00", dec!(0.00).to_string());
    assert_eq!("1.00", dec!(1.00).to_string());
    assert_eq!("-1.23", dec!(-1.23).to_string());
    assert_eq!("1.1234567890123456789012345678", 
        dec!(1.1234567890123456789012345678).to_string());
}