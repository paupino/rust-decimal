use core::str::FromStr;
use num_traits::One;
use rust_decimal::{Decimal, MathematicalOps};

#[test]
fn test_constants() {
    assert_eq!("3.1415926535897932384626433833", Decimal::PI.to_string());
    assert_eq!("6.2831853071795864769252867666", Decimal::TWO_PI.to_string());
    assert_eq!("1.5707963267948966192313216916", Decimal::HALF_PI.to_string());
    assert_eq!("2.7182818284590452353602874714", Decimal::E.to_string());
    assert_eq!("0.3678794411714423215955237702", Decimal::E_INVERSE.to_string());
}

#[test]
fn test_powu() {
    let test_cases = &[
        // x, y, expected x ^ y
        ("4", 3_u64, "64"),
        ("3.222", 5_u64, "347.238347228449632"),
        ("0.1", 0_u64, "1"),
        ("342.4", 1_u64, "342.4"),
        ("2.0", 16_u64, "65536"),
        ("0.99999999999999", 1477289400_u64, "0.9999852272151186611602878472"),
        ("0.99999999999999", 0x8000_8000_0000_0000, "0"),
    ];
    for &(x, y, expected) in test_cases {
        let x = Decimal::from_str(x).unwrap();
        let pow = x.powu(y);
        assert_eq!(pow.to_string(), expected, "{} ^ {}", x, y);
    }
}

#[test]
#[should_panic(expected = "Pow overflowed")]
fn test_powu_panic() {
    let two = Decimal::new(2, 0);
    let _ = two.powu(128);
}

#[test]
fn test_checked_powu() {
    let test_cases = &[
        (Decimal::new(4, 0), 3_u64, Some(Decimal::new(64, 0))),
        (
            Decimal::from_str("3.222").unwrap(),
            5_u64,
            Some(Decimal::from_str("347.238347228449632").unwrap()),
        ),
        (
            Decimal::from_str("0.1").unwrap(),
            0_u64,
            Some(Decimal::from_str("1").unwrap()),
        ),
        (
            Decimal::from_str("342.4").unwrap(),
            1_u64,
            Some(Decimal::from_str("342.4").unwrap()),
        ),
        (
            Decimal::from_str("2.0").unwrap(),
            16_u64,
            Some(Decimal::from_str("65536").unwrap()),
        ),
        (Decimal::from_str("2.0").unwrap(), 128_u64, None),
    ];
    for case in test_cases {
        assert_eq!(case.2, case.0.checked_powu(case.1));
    }
}

#[test]
fn test_powi() {
    let test_cases = &[
        // x, y, expected x ^ y
        ("0", 0, "1"),
        ("1", 0, "1"),
        ("0", 1, "0"),
        ("2", 3, "8"),
        ("-2", 3, "-8"),
        ("2", -3, "0.125"),
        ("-2", -3, "-0.125"),
        ("3", -3, "0.037037037037037037037037037"),
        ("6", 3, "216"),
        ("0.5", 2, "0.25"),
    ];
    for &(x, y, expected) in test_cases {
        let x = Decimal::from_str(x).unwrap();
        let pow = x.powi(y);
        assert_eq!(pow.to_string(), expected, "{} ^ {}", x, y);
    }
}

#[test]
fn test_powd() {
    let test_cases = &[
        // x, y, expected x ^ y
        ("0", "0", "1"),
        ("1", "0", "1"),
        ("0", "1", "0"),
        ("2", "3", "8"),
        ("-2", "3", "-8"),
        ("2", "-3", "0.125"),
        ("-2", "-3", "-0.125"),
        ("2.0", "3.0", "8"),
        ("-2.0", "3.0", "-8"),
        ("2.0", "-3.0", "0.125"),
        ("-2.0", "-3.0", "-0.125"),
        ("2.00", "3.00", "8"),
        ("-2.00", "3.00", "-8"),
        ("2.00", "-3.00", "0.125"),
        ("-2.00", "-3.00", "-0.125"),
        ("3", "-3", "0.037037037037037037037037037"),
        ("6", "3", "216"),
        ("0.5", "2", "0.25"),
        ("6", "13", "13060694016"),
        // Exact result: 1 / 6^7
        ("6", "-7", "0.0000035722450845907636031093"),
        ("0.16", "12.5", "0.0000000001125899906842624"),
        // ~= 0.8408964152537145
        ("0.5", "0.25", "0.8408964152537145430311254764"),
        // ~= 0.999999999999999999999999999790814
        (
            "0.1234567890123456789012345678",
            "0.0000000000000000000000000001",
            "0.9999999999999999999999999998",
        ),
        ("1234.5678", "0.9012", "611.04510432242565327379383059"),
        ("-2", "0.5", "-1.4142135623730950488016887244"),
        ("-2.5", "0.123", "-1.1193003023312942509616251298"),
        (
            "0.0000000000000000000000000001",
            "0.1234567890123456789012345678",
            "0.0003493091063371512550217181",
        ),
        ("0.99999999999999", "1477289400", "0.9999852272151186611602878472"),
    ];
    for &(x, y, expected) in test_cases {
        let x = Decimal::from_str(x).unwrap();
        let y = Decimal::from_str(y).unwrap();
        let pow = x.powd(y);
        assert_eq!(pow.to_string(), expected, "{} ^ {}", x, y);
    }
}

#[test]
fn test_sqrt() {
    let test_cases = &[
        ("4", "2"),
        ("3.222", "1.7949930361981909371487724124"),
        ("199.45", "14.122676800097069416754994263"),
        ("342.4", "18.504053609952604112132102540"),
        ("2", "1.414213562373095048801688724209698078569671875376948073176"),
        ("0.0000000000000000000000000001", "0.0000000000000100000000000000"),
    ];
    for case in test_cases {
        let a = Decimal::from_str(case.0).unwrap();
        let expected = Decimal::from_str(case.1).unwrap();
        assert_eq!(expected, a.sqrt().unwrap());
    }

    assert_eq!(Decimal::new(-2, 0).sqrt(), None);
}

#[test]
fn test_exp() {
    let test_cases = &[
        ("20", "485165195.40979027796910683072"),
        ("10", "22026.46579480671651695790065"),
        ("11", "59874.141715197818455326485804"),
        ("3", "20.085536923187667740928529656"),
        ("8", "2980.9579870417282747435920999"),
        ("0.1", "1.1051709180756476248117078265"),
        ("2.0", "7.3890560989306502272304274609"),
        ("-2", "0.135335283236612691893999495"),
        ("-1", "0.3678794411714423215955237702"),
        ("0.123456789", "1.1314011145122336038009871011"),
        ("0.123456789123456789123456789", "1.1314011146519127526179900813"),
        // low-scale inputs: previously wrong by ~8 digits, now within 1 ULP
        ("0.5", "1.6487212707001281468486507879"),
    ];
    for &(x, expected) in test_cases {
        let x = Decimal::from_str(x).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        assert_eq!(expected, x.exp());
        assert_eq!(Some(expected), x.checked_exp());
    }
}

#[test]
fn test_exp_with_tolerance() {
    let test_cases = &[
        // e^0 = 1
        ("0", "0.0002", "1"),
        // e^1 ~= 2.7182539682539682539682539683
        ("1", "0.0002", "2.7182539682539682539682539683"),
        // e^10 ~= 22026.4657948067165169579006453
        ("10", "0.0002", "22026.465747586389870727226329"),
        // e^11 ~= 59874.1417151978184553264857923
        ("11", "0.0002", "59874.141680567932663075014271"),
        // e^11.7578 ~= 127746.03548949540892948423052
        ("11.7578", "0.0002", "127746.10261578321551805466515"),
        // e^3 ~= 20.085536923187664
        ("3", "0.00002", "20.085534430970814899386327956"),
        // e^8 ~= 2980.957987041727
        ("8", "0.0002", "2980.9579632726949323840240320"),
        // e^0.1 ~= 1.1051709180756477
        ("0.1", "0.0002", "1.1051666666666666666666666667"),
        // e^2.0 ~= 7.3890560989306495
        ("2.0", "0.0002", "7.3890460157126823793490460164"),
        // e^66.542 ~= 7.92179e28
        ("66.542", "0.0002", "79217916301129338946322758261"),
        // e^66.543 overflows
        ("66.543", "0.0002", ""),
        // e^123 overflows
        ("123", "0.0002", ""),
        // e^-8 ~= 0.000335462627902511838821389125781
        ("-8", "0.0002", "0.0003354626305773641802399811"),
        // e^-9 ~= 0.000123409804086679549497636690730
        ("-9", "0.0002", "0.0001234098050655108657269324"),
        // e^-11 ~= 0.0000167017007902456593126355173606
        ("-11", "0.0002", "0.0000167017007999055554659406"),
        // e^66.542 ~= 1.26234*10^-29
        ("-66.542", "0.0002", "0"),
        // e^66.543 underflows
        ("-66.543", "0.0002", ""),
        // e^-1024 underflows
        ("-1024", "0.0002", ""),
    ];
    for &(x, tolerance, expected) in test_cases {
        let x = Decimal::from_str(x).unwrap();
        let tolerance = Decimal::from_str(tolerance).unwrap();
        let expected = if expected.is_empty() {
            None
        } else {
            Some(Decimal::from_str(expected).unwrap())
        };

        if let Some(expected) = expected {
            assert_eq!(expected, x.exp_with_tolerance(tolerance));
            assert_eq!(Some(expected), x.checked_exp_with_tolerance(tolerance));
        } else {
            assert_eq!(None, x.checked_exp_with_tolerance(tolerance));
        }
    }
}

#[test]
#[should_panic(expected = "Exp overflowed")]
fn test_exp_expected_panic_from_overflow() {
    let d = Decimal::from_str("1024").unwrap();
    let _ = d.exp();
}

#[test]
#[should_panic(expected = "Exp underflowed")]
fn test_exp_expected_panic_from_underflow() {
    let d = Decimal::from_str("-1024").unwrap();
    let _ = d.exp();
}

#[test]
fn test_norm_cdf() {
    let test_cases = &[
        (
            Decimal::from_str("-0.4").unwrap(),
            Decimal::from_str("0.3445781286821245037094401704").unwrap(),
        ),
        (
            Decimal::from_str("-0.1").unwrap(),
            Decimal::from_str("0.4601722899186706579921922696").unwrap(),
        ),
        (
            Decimal::from_str("0.1").unwrap(),
            Decimal::from_str("0.5398277100813293420078077304").unwrap(),
        ),
        (
            Decimal::from_str("0.4").unwrap(),
            Decimal::from_str("0.6554218713178754962905598296").unwrap(),
        ),
        (
            Decimal::from_str("2.0").unwrap(),
            Decimal::from_str("0.9772497381095865280953380673").unwrap(),
        ),
    ];
    for case in test_cases {
        assert_eq!(case.1, case.0.norm_cdf());
    }
}

#[test]
fn test_norm_pdf() {
    let test_cases = &[
        (
            Decimal::from_str("-2.0").unwrap(),
            Decimal::from_str("0.0539909665131880519505642004").unwrap(),
        ),
        (
            Decimal::from_str("-0.4").unwrap(),
            Decimal::from_str("0.3682701403033233077441296229").unwrap(),
        ),
        (
            Decimal::from_str("-0.1").unwrap(),
            Decimal::from_str("0.3969525474770117655105297435").unwrap(),
        ),
        (
            Decimal::from_str("0.1").unwrap(),
            Decimal::from_str("0.3969525474770117655105297435").unwrap(),
        ),
        (
            Decimal::from_str("0.4").unwrap(),
            Decimal::from_str("0.3682701403033233077441296229").unwrap(),
        ),
        (
            Decimal::from_str("2.0").unwrap(),
            Decimal::from_str("0.0539909665131880519505642004").unwrap(),
        ),
    ];
    for case in test_cases {
        assert_eq!(case.1, case.0.norm_pdf());
    }
}

#[test]
fn test_ln() {
    let test_cases = [
        ("1", "0"),
        ("0.23", "-1.4696759700589416772292300767"),
        ("2", "0.6931471805599453094172321216"),
        ("25", "3.2188758248682007492015186659"),
        ("1.234567890", "0.2107210222156525610500017105"),
        // tiny values: range reduction via narrow ×E compounds rounding errors
        ("0.0000000000000000000000000001", "-64.472382603833279152503760712"),
        ("0.0000000000000000000000000007", "-62.526472454777965847398407968"),
    ];

    for (input, expected) in test_cases {
        let input = Decimal::from_str(input).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        assert_eq!(expected, input.ln(), "Failed to calculate ln({})", input);
    }
}

#[test]
#[cfg(feature = "maths-nopanic")]
fn test_invalid_ln_nopanic() {
    let test_cases = ["0", "-2.0"];

    for input in test_cases {
        let input = Decimal::from_str(input).unwrap();
        assert_eq!("0", input.ln().to_string(), "Failed to calculate ln({})", input);
    }
}

#[test]
#[should_panic(expected = "Unable to calculate ln for zero")]
#[cfg(not(feature = "maths-nopanic"))]
fn test_invalid_ln_zero_panic() {
    let _ = Decimal::ZERO.ln();
}

#[test]
#[should_panic(expected = "Unable to calculate ln for negative numbers")]
#[cfg(not(feature = "maths-nopanic"))]
fn test_invalid_ln_negative_panic() {
    let _ = Decimal::NEGATIVE_ONE.ln();
}

#[test]
fn test_log10() {
    let test_cases = [
        ("1", "0"),
        ("2", "0.3010299956639811952137388948"),
        ("1.234567890", "0.0915149771692704475183336231"),
        ("10", "1"),
        ("100", "2"),
        ("1000", "3"),
        ("1000.00000000", "3"),
        ("1000.000000000000000000000", "3"),
        ("10.000000000000000000000000000", "1"),
        ("100000000000000.0000000000", "14"),
        ("0.10", "-1"),
        ("0.1", "-1"),
        ("0.01", "-2"),
        ("0.010", "-2"),
        ("0.0100000000000000000", "-2"),
        ("0.000001", "-6"),
        ("0.000001000000000", "-6"),
    ];

    for (input, expected) in test_cases {
        let input = Decimal::from_str(input).unwrap();
        let expected = Decimal::from_str(expected).unwrap();
        assert_eq!(expected, input.log10(), "Failed to calculate log10({})", input);
    }
}

#[test]
#[cfg(feature = "maths-nopanic")]
fn test_invalid_log10_nopanic() {
    let test_cases = ["0", "-2.0"];

    for input in test_cases {
        let input = Decimal::from_str(input).unwrap();
        assert_eq!("0", input.log10().to_string(), "Failed to calculate ln({})", input);
    }
}

#[test]
#[should_panic(expected = "Unable to calculate log10 for zero")]
#[cfg(not(feature = "maths-nopanic"))]
fn test_invalid_log10_zero_panic() {
    let _ = Decimal::ZERO.log10();
}

#[test]
#[should_panic(expected = "Unable to calculate log10 for negative numbers")]
#[cfg(not(feature = "maths-nopanic"))]
fn test_invalid_log10_negative_panic() {
    let _ = Decimal::NEGATIVE_ONE.log10();
}

#[test]
fn test_erf() {
    let test_cases = &[
        (
            Decimal::from_str("-2.0").unwrap(),
            // Wolfram give -0.9953222650189527
            Decimal::from_str("-0.9953225170750043399400930073").unwrap(),
        ),
        (
            Decimal::from_str("-0.4").unwrap(),
            Decimal::from_str("-0.428392412720515497796193142").unwrap(),
        ),
        (
            Decimal::from_str("0.4").unwrap(),
            Decimal::from_str("0.428392412720515497796193142").unwrap(),
        ),
        (
            Decimal::one(),
            Decimal::from_str("0.8427010463338918630217928957").unwrap(),
        ),
        (
            Decimal::from_str("2").unwrap(),
            Decimal::from_str("0.9953225170750043399400930073").unwrap(),
        ),
    ];
    for case in test_cases {
        assert_eq!(case.1, case.0.erf());
    }
}

#[test]
fn test_checked_sin() {
    const ACCEPTED_PRECISION: u32 = 10;
    let test_cases = &[
        // Sin(0)
        ("0", Some("0")),
        // Sin(PI/2)
        ("1.5707963267948966192313216916", Some("1")),
        // Sin(PI)
        ("3.1415926535897932384626433833", Some("0")),
        // Sin(3PI/2)
        ("4.7123889803846898576939650749", Some("-1")),
        // Sin(2PI)
        ("6.2831853071795864769252867666", Some("0")),
        // Sin(1) ~= 0.8414709848078965066525023216302989996225630607983710656727517099
        ("1", Some("0.8414709848078965066525023216")),
        // Sin(2) ~= 0.9092974268256816953960198659117448427022549714478902683789730115
        ("2", Some("0.9092974268256816953960198659")),
        // Sin(4) ~= -0.756802495307928251372639094511829094135912887336472571485416773
        ("4", Some("-0.7568024953079282513726390945")),
        // Sin(6) ~= -0.279415498198925872811555446611894759627994864318204318483351369
        ("6", Some("-0.2794154981989258728115554466")),
        // WA estimate: -0.893653245236708. Legacy ops is closer to f64 accuracy.
        ("-79228162514264.337593543950335", Some("-0.893653245236708")),
        ("0.7853981633974483096156608458", Some("0.7071067811865475244008443621")),
    ];
    for (input, result) in test_cases {
        let radians = Decimal::from_str(input).unwrap();
        let sin = radians.checked_sin();
        if let Some(result) = result {
            assert!(sin.is_some(), "Expected result for sin({})", input);
            let result = Decimal::from_str(result).unwrap();
            {
                let left = sin.unwrap();
                let right = result;
                let eps = Decimal::new(1, ACCEPTED_PRECISION);
                let diff = (left - right).abs();
                assert!(
                    diff <= eps,
                    "sin({}): assertion failed: `(left ~= right)`\n  left: `{:?}`\n right: `{:?}`\n   eps: `{:?}`\n  diff: `{:?}`",
                    input,
                    left,
                    right,
                    eps,
                    diff,
                );
            }
        } else {
            assert!(sin.is_none(), "Unexpected result for sin({})", input);
        }
    }
}

#[test]
fn test_checked_cos() {
    const ACCEPTED_PRECISION: u32 = 10;
    let test_cases = &[
        // Cos(0)
        ("0", Some("1")),
        // Cos(PI/2)
        ("1.5707963267948966192313216916", Some("0")),
        // Cos(PI)
        ("3.1415926535897932384626433833", Some("-1")),
        // Cos(3PI/2)
        ("4.7123889803846898576939650749", Some("0")),
        // Cos(2PI)
        ("6.2831853071795864769252867666", Some("1")),
        // Cos(1) ~= 0.5403023058681397174009366074429766037323104206179222276700972553
        ("1", Some("0.5403023058681397174009366074")),
        // Cos(2) ~= -0.416146836547142386997568229500762189766000771075544890755149973
        ("2", Some("-0.4161468365471423869975682295")),
        // Cos(4) ~= -0.653643620863611914639168183097750381424133596646218247007010283
        ("4", Some("-0.6536436208636119146391681831")),
        // Cos(6) ~= 0.9601702866503660205456522979229244054519376792110126981292864260
        ("6", Some("0.9601702866503660205456522979")),
        // WA estimate: 0.448758150096352. Legacy ops is closer to f64 accuracy.
        ("-79228162514264.337593543950335", Some("0.448758150096352")),
        ("0.7853981633974483096156608458", Some("0.7071067811865475244008443622")),
        ("8.639379797371931405772269304", Some("-0.7071067811865475244008443621")),
    ];
    for (input, result) in test_cases {
        let radians = Decimal::from_str(input).unwrap();
        let cos = radians.checked_cos();
        if let Some(result) = result {
            assert!(cos.is_some(), "Expected result for cos({})", input);
            let result = Decimal::from_str(result).unwrap();
            {
                let left = cos.unwrap();
                let right = result;
                let eps = Decimal::new(1, ACCEPTED_PRECISION);
                let diff = (left - right).abs();
                assert!(
                    diff <= eps,
                    "cos({}): assertion failed: `(left ~= right)`\n  left: `{:?}`\n right: `{:?}`\n   eps: `{:?}`\n  diff: `{:?}`",
                    input,
                    left,
                    right,
                    eps,
                    diff,
                );
            }
        } else {
            assert!(cos.is_none(), "Unexpected result for cos({})", input);
        }
    }
}

#[test]
fn test_checked_tan() {
    const ACCEPTED_PRECISION: u32 = 8;
    let test_cases = &[
        // Tan(0)
        ("0", Some("0")),
        // Tan(PI/2)
        ("1.5707963267948966192313216916", None),
        // Tan(PI)
        ("3.1415926535897932384626433833", Some("0")),
        // Tan(3PI/2)
        ("4.7123889803846898576939650749", None),
        // Tan(2PI)
        ("6.2831853071795864769252867666", Some("0")),
        // Tan(1) ~= 1.5574077246549022305069748074583601730872507723815200383839466056
        ("1", Some("1.5574077246549022305069748075")),
        // Tan(2) ~= -2.185039863261518991643306102313682543432017746227663164562955869
        ("2", Some("-2.1850398632615189916433061023")),
        // Tan(4) ~= 1.1578212823495775831373424182673239231197627673671421300848571893
        ("4", Some("1.1578212823495775831373424183")),
        // Tan(6) ~= -0.291006191384749157053699588868175542831155570912339131608827193
        ("6", Some("-0.2910061913847491570536995889")),
        // WA estimate: -1.99139167733184. Legacy ops is closer to f64 accuracy.
        ("-79228162514264.337593543950335", Some("-1.99139167733184")),
    ];
    for (input, result) in test_cases {
        let radians = Decimal::from_str(input).unwrap();
        let tan = radians.checked_tan();
        if let Some(result) = result {
            assert!(tan.is_some(), "Expected result for tan({})", input);
            let result = Decimal::from_str(result).unwrap();
            {
                let left = tan.unwrap();
                let right = result;
                let eps = Decimal::new(1, ACCEPTED_PRECISION);
                let diff = (left - right).abs();
                assert!(
                    diff <= eps,
                    "tan({}): assertion failed: `(left ~= right)`\n  left: `{:?}`\n right: `{:?}`\n   eps: `{:?}`\n  diff: `{:?}`",
                    input,
                    left,
                    right,
                    eps,
                    diff,
                );
            }
        } else {
            assert!(tan.is_none(), "Unexpected result for tan({})", input);
        }
    }
}
