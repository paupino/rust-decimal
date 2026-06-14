use rust_decimal::prelude::*;

#[test]
fn issue_384_neg_overflow_during_subtract_carry() {
    // 288230376151711744
    let a = Decimal::from_parts(0, 67108864, 0, false, 0);
    // 714606955844629274884780.85120
    let b = Decimal::from_parts(0, 0, 3873892070, false, 5);
    let c = a.checked_sub(b);
    assert!(c.is_some());

    //         288230376151711744.
    // - 714606955844629274884780.85120
    // =
    // - 714606667614253123173036.85120
    assert_eq!("-714606667614253123173036.85120", c.unwrap().to_string());
}

#[test]
fn subtract_borrow_propagation_into_upper_words() {
    // In `unaligned_add`'s subtract path the borrow-propagation loop had an
    // inverted stop condition, so when a large integer has a small high-scale
    // fraction subtracted from it (the scaled operand spilling into the Buf24
    // upper words with bits 96..128 equal to 0 or 1) the borrow out of the low
    // 96 bits was either dropped or applied spuriously, leaving the result
    // wrong by ~2^128.
    fn sub(a: &str, b: &str, expected: &str) {
        let a = Decimal::from_str(a).unwrap();
        let b = Decimal::from_str(b).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        // Value-equality is scale-independent, which is what we care about here.
        assert_eq!(expected, a - b, "{a} - {b}");
    }

    // word3 == 0: borrow was lost -> result ~2^128 too large.
    sub("34028236693", "7.0000000000000000000000000000", "34028236686");
    // word3 == 1: spurious borrow -> result ~2^128 too small.
    sub("34028236701", "7.0000000000000000000000000000", "34028236694");
}

#[test]
fn issue_392_overflow_during_remainder() {
    let a = Decimal::from_str("-79228157791897.854723898738431").unwrap();
    let b = Decimal::from_str("184512476.73336922111").unwrap();
    let c = a.checked_div(b);
    assert!(c.is_some());
    assert_eq!("-429391.87200000000002327170816", c.unwrap().to_string())
}

#[test]
fn issue_624_to_f64_precision() {
    let tests = [
        ("1000000.000000000000000000", 1000000.0f64),
        ("10000.000000000000000000", 10000.0f64),
        ("100000.000000000000000000", 100000.0f64), // Problematic value
    ];
    for (index, (test, expected)) in tests.iter().enumerate() {
        let decimal = Decimal::from_str_exact(test).unwrap();
        assert_eq!(
            f64::try_from(decimal).unwrap(),
            *expected,
            "Test index {} failed",
            index
        );
    }
}

#[test]
fn issue_618_rescaling_overflow() {
    fn assert_result(scale: u32, v1: Decimal, v2: Decimal) {
        assert_eq!(scale, v1.scale(), "initial scale: {scale}");
        let result1 = v1 + -v2;
        assert_eq!(
            result1.to_string(),
            "-0.0999999999999999999999999999",
            "a + -b : {scale}"
        );
        assert_eq!(28, result1.scale(), "a + -b : {scale}");
        let result2 = v1 - v2;
        assert_eq!(
            result2.to_string(),
            "-0.0999999999999999999999999999",
            "a - b : {scale}"
        );
        assert_eq!(28, result2.scale(), "a - b : {scale}");
    }

    let mut a = Decimal::from_str("0.0000000000000000000000000001").unwrap();
    let b = Decimal::from_str("0.1").unwrap();
    assert_result(28, a, b);

    // Try at a new scale (this works)
    a.rescale(30);
    assert_result(30, a, b);

    // And finally the scale causing an issue
    a.rescale(29);
    assert_result(29, a, b);
}
