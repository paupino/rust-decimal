extern crate rust_decimal;
extern crate rust_decimal_macros;

use rust_decimal_macros::dec;

#[test]
fn it_can_parse_decimal() {
    let tests = &[
        ("0.00", dec!(0.00)),
        ("1.00", dec!(1.00)),
        ("-1.23", dec!(-1.23)),
        ("1.1234567890123456789012345678", dec!(1.1234567890123456789012345678)),
        ("1000000", dec!(1_000_000)),
    ];
    for &(a, b) in tests {
        assert_eq!(a, b.to_string());
    }
}
