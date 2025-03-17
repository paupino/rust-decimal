// TODO: Make all of the new tests work with the new parameters etc
use rust_decimal_macros::dec;

// Require using for reexportable feature
#[cfg(feature = "reexportable")]
use rust_decimal::Decimal;

#[test]
fn it_can_parse_standard_decimal() {
    let tests = &[
        (dec!(0.00), "0.00"),
        (dec!(1.00), "1.00"),
        (dec!(1.23), "1.23"),
        (dec!(-1.23), "-1.23"),
        (dec!(1.1234567890123456789012345678), "1.1234567890123456789012345678"),
        (dec!(1_000_000), "1000000"),
        (dec!(1), "1"),
        (dec!(-1), "-1"),
        (dec!(1_999), "1999"),
        (dec!(-1_999), "-1999"),
        (dec!(1.), "1"),
        (dec!(-1.111_009), "-1.111009"),
    ];
    for &(a, b) in tests {
        assert_eq!(a.to_string(), b);
    }
}

#[test]
fn it_can_parse_alternative_base_decimal() {
    let tests = &[
        (dec!(0b1), "1"),
        (dec!(-0b1_1111), "-31"),
        (dec!(0o1), "1"),
        (dec!(-0o1_777), "-1023"),
        (dec!(0x1), "1"),
        (dec!(-0x1_Ffff), "-131071"),
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
        (dec!(1e6), "1000000"),
        (dec!(-1.2e+6), "-1200000"),
        (dec!(12e-6), "0.000012"),
        (dec!(-1.2e-6), "-0.0000012"),
    ];
    for &(a, b) in tests {
        assert_eq!(a.to_string(), b);
    }
}

#[test]
fn it_can_parse_decimal_with_args() {
    let tests = &[
        (dec!(radix: 2, 100), "4"),
        (dec!(radix: 3, -1_222), "-53"),
        (dec!(radix: 36, z1), "1261"),
        (dec!(radix: 36, -1_xyz), "-90683"),
        (dec!(radix: 2, exp: 5, 10), "200000"),
        (dec!(exp: -3, radix: 8, -1_777), "-1.023"),
        (dec!(exp: -3, -1023), "-1.023"),
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

#[test]
// dec!() macro and old parser give same result
pub fn dec_exact() {
    macro_rules! test {
        ($src:literal) => {
            assert_eq!(
                dec!($src),
                rust_decimal::Decimal::from_str_exact(stringify!($src)).unwrap(),
                stringify!($src)
            );
        };
    }
    test!(1_000);
    test!(-1_000);
    test!(0.000_001);
    test!(-0.000_001);
    test!(79_228_162_514_264_337_593_543_950_335);
    test!(-79_228_162_514_264_337_593_543_950_335);
    test!(79.228_162_514_264_337_593_543_950_335);
    test!(-79.228_162_514_264_337_593_543_950_335);
    test!(7.922_816_251_426_433_759_354_395_033_5);
    test!(-7.922_816_251_426_433_759_354_395_033_5);
}

#[test]
// dec!() macro and old parser give same result
pub fn dec_scientific() {
    macro_rules! test {
        ($src:literal) => {
            assert_eq!(
                dec!($src),
                rust_decimal::Decimal::from_scientific(stringify!($src)).unwrap(),
                stringify!($src)
            );
        };
    }
    test!(1e1);
    test!(-1e1);
    test!(1e+1);
    test!(-1e+1);
    test!(1e-1);
    test!(-1e-1);

    test!(1.1e1);
    test!(-1.1e1);
    test!(1.1e+1);
    test!(-1.1e+1);
    test!(1.1e-1);
    test!(-1.1e-1);

    test!(7.922_816_251_426_433_759_354_395_033_5e28);
    test!(-7.922_816_251_426_433_759_354_395_033_5e28);
    test!(79_228_162_514_264_337_593_543_950_335e-28);
    test!(-79_228_162_514_264_337_593_543_950_335e-28);
}
