use rust_decimal_macros::dec;

#[test]
fn it_can_parse_decimal() {
    let tests = &[
        ("0.00", dec!(0.00)),
        ("1.00", dec!(1.00)),
        ("-1.23", dec!(-1.23)),
        ("1.1234567890123456789012345678", dec!(1.1234567890123456789012345678)),
        ("1000000", dec!(1_000_000)),
        ("123", dec!(1.23e2)),
        ("123", dec!(1.23e+2)),
        ("-0.0123", dec!(-1.23e-2)),
        ("3.14", dec!(3.14e0)),
        ("12000", dec!(12e3)),
    ];
    for &(a, b) in tests {
        assert_eq!(a, b.to_string());
    }
}
