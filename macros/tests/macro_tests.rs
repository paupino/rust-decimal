use rust_decimal_macros::dec;

// Require using for reexportable feature
#[cfg(feature = "reexportable")]
use rust_decimal::Decimal;

#[test]
fn it_can_parse_standard_decimal() {
    let tests = &[
        (dec!(0.00), "0.00"),
        (dec!(1.00), "1.00"),
        (dec!(-1.23), "-1.23"),
        (dec!(1.1234567890123456789012345678), "1.1234567890123456789012345678"),
        (dec!(1_000_000), "1000000"),
    ];
    for &(a, b) in tests {
        assert_eq!(a.to_string(), b);
    }
}

#[test]
fn it_can_parse_scientific_decimal() {
    let tests = &[
        (dec!(1.23e2), "123"),
        (dec!(1.23e+2), "123"),
        (dec!(-1.23e-2), "-0.0123"),
        (dec!(3.14e0), "3.14"),
        (dec!(12e3), "12000"),
        (dec!(9.7e-7), "0.00000097"),
        (dec!(9e-7), "0.0000009"),
        (dec!(1.2e10), "12000000000"),
        (dec!(1.2e+10), "12000000000"),
        (dec!(12e10), "120000000000"),
        (dec!(9.7E-7), "0.00000097"),
        (dec!(1.2345E-24), "0.0000000000000000000000012345"),
        (dec!(12345E-28), "0.0000000000000000000000012345"),
        (dec!(1.2345E0), "1.2345"),
        (dec!(1E28), "10000000000000000000000000000"),
    ];
    for &(a, b) in tests {
        assert_eq!(a.to_string(), b);
    }
}

#[test]
fn invalid_input() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/invalid/*.rs");
}
