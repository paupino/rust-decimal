use std::fmt::{Display, Formatter};

const MAX_I128_REPR: i128 = 0x0000_0000_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF;

#[derive(Debug)]
pub struct Unpacked {
    pub mantissa: i128,
    pub scale: u32,
}

impl Unpacked {
    pub const fn lo(&self) -> u32 {
        self.mantissa.unsigned_abs() as u32
    }

    pub const fn mid(&self) -> u32 {
        (self.mantissa.unsigned_abs() >> 32) as u32
    }

    pub const fn hi(&self) -> u32 {
        (self.mantissa.unsigned_abs() >> 64) as u32
    }

    pub const fn negative(&self) -> bool {
        self.mantissa < 0
    }
}

pub type ParseResult<'str> = Result<Unpacked, ParseError<'str>>;

#[derive(Debug, PartialEq)]
pub enum ParseError<'src> {
    Empty,
    ExceedsMaximumPossibleValue,
    FractionEmpty,
    InvalidExp(i32),
    InvalidRadix(u32),
    LessThanMinimumPossibleValue,
    Underflow,
    Unparseable(&'src [u8]),
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "Number is empty, must have an integer part."),
            ParseError::ExceedsMaximumPossibleValue => {
                write!(f, "Number exceeds maximum value that can be represented.")
            }
            ParseError::FractionEmpty => write!(f, "Fraction empty, consider adding a `0` after the period."),
            ParseError::InvalidExp(exp) if *exp < 0 => write!(
                f,
                "Scale exceeds the maximum precision allowed: {} > {}.",
                exp.unsigned_abs(),
                28
            ),
            ParseError::InvalidExp(exp) => {
                write!(f, "Invalid exp {exp} -- exp must be in the range -28 to 28 inclusive.")
            }
            ParseError::InvalidRadix(radix) => write!(
                f,
                "Invalid radix {radix} -- radix must be in the range 2 to 36 inclusive."
            ),
            ParseError::LessThanMinimumPossibleValue => {
                write!(f, "Number less than minimum value that can be represented.")
            }
            ParseError::Underflow => write!(f, "Number has a high precision that can not be represented."),
            ParseError::Unparseable(src) => {
                write!(
                    f,
                    "Cannot parse decimal, unexpected \"{}\".",
                    core::str::from_utf8(src).unwrap()
                )
            }
        }
    }
}

// dec!() entrypoint without radix
pub const fn parse_dec(src: &str, exp: i32) -> ParseResult {
    const fn skip_us(radix: u32, is_positive: bool, mut src: &[u8], exp: i32) -> ParseResult {
        while let [b'_', rest @ ..] = src {
            src = rest
        }
        parse_bytes(radix, is_positive, src, exp)
    }

    let (is_positive, src) = parse_sign(src);
    match src {
        [b'0', b'b', src @ ..] => skip_us(2, is_positive, src, exp),
        [b'0', b'o', src @ ..] => skip_us(8, is_positive, src, exp),
        [b'0', b'x', src @ ..] => skip_us(16, is_positive, src, exp),
        src => parse_10(is_positive, src, exp),
    }
}

// dec!() entrypoint with radix
pub const fn parse_radix_dec(radix: u32, src: &str, exp: i32) -> ParseResult {
    if 2 <= radix && radix <= 36 {
        let (is_positive, src) = parse_sign(src);
        parse_bytes(radix, is_positive, src, exp)
    } else {
        Err(ParseError::InvalidRadix(radix))
    }
}

const fn parse_sign(src: &str) -> (bool, &[u8]) {
    let mut src = src.as_bytes();
    if let [b'-', signed @ ..] = src {
        src = signed;
        while let [b' ' | b'\t' | b'\n', rest @ ..] = src {
            src = rest;
        }
        (false, src)
    } else {
        (true, src)
    }
}

const fn parse_bytes(radix: u32, is_positive: bool, src: &[u8], exp: i32) -> ParseResult {
    match parse_bytes_inner(radix, src, 0) {
        (.., Some(rest)) => Err(ParseError::Unparseable(rest)),
        (_, 0, _) => Err(ParseError::Empty),
        (num, ..) => to_decimal(is_positive, num, exp),
    }
}

// translate bi-directional exp to rest of lib’s scale down-only and create Decimal
const fn to_decimal<'src>(is_positive: bool, mut num: i128, mut exp: i32) -> ParseResult<'src> {
    // Why is scale unsigned? :-(
    use ParseError::*;

    const POWERS_10: [i128; 29] = {
        let mut powers_10 = [1; 29];
        // no iter in const
        let mut i = 1;
        while i < 29 {
            powers_10[i] = 10 * powers_10[i - 1];
            i += 1;
        }
        powers_10
    };

    if exp >= 0 {
        if exp > 28 {
            return Err(InvalidExp(exp));
        }
        if let Some(shifted) = num.checked_mul(POWERS_10[exp as usize]) {
            num = shifted
        }
        exp = 0;
    } else if exp < -28 {
        return Err(InvalidExp(exp));
    }
    if num > MAX_I128_REPR {
        return Err(if is_positive {
            ExceedsMaximumPossibleValue
        } else {
            LessThanMinimumPossibleValue
        });
    }
    Ok(Unpacked {
        mantissa: if is_positive { num } else { -num },
        scale: exp.unsigned_abs(),
    })
}

// parse normal (radix 10) numbers with optional float-like .fraction and 10’s exponent
const fn parse_10(is_positive: bool, src: &[u8], mut exp: i32) -> ParseResult {
    // parse 1st part (upto optional . or e)
    let (mut num, len, mut more) = parse_bytes_inner(10, src, 0);
    // Numbers can’t be empty (before optional . or e)
    if len == 0 {
        return Err(ParseError::Empty);
    }

    // parse optional fraction
    if let Some([b'.', rest @ ..]) = more {
        let (whole_num, scale, _more) = parse_bytes_inner(10, rest, num);
        more = _more;
        // May only be empty if no exp
        if scale == 0 && more.is_some() {
            return Err(ParseError::FractionEmpty);
        }
        num = whole_num;
        if num > MAX_I128_REPR {
            return Err(ParseError::Underflow);
        }
        exp -= scale as i32
    }

    // parse optional 10’s exponent
    if let Some([b'e' | b'E', rest @ ..]) = more {
        let (rest, exp_is_positive) = if let [sign @ b'-' | sign @ b'+', signed @ ..] = rest {
            (signed, *sign == b'+')
        } else {
            (rest, true)
        };
        // if this gives Some more, we’ll return that below
        let (e_part, _, _more) = parse_bytes_inner(10, rest, 0);
        more = _more;
        // dummy value, more than MAX not storable
        if e_part > i32::MAX as i128 {
            return Err(ParseError::InvalidExp(i32::MAX));
        }
        if exp_is_positive {
            exp += e_part as i32
        } else {
            exp -= e_part as i32
        }
    }

    if let Some(rest) = more {
        Err(ParseError::Unparseable(rest))
    } else {
        to_decimal(is_positive, num, exp)
    }
}

// Can’t use `from_str_radix`, as that neither groks '_', nor allows to continue after '.' or 'e'.
// For multi-step (see test) return: number parsed, digits count, offending rest
// num saturates at i128::MAX, which is currently not a valid Decimal
const fn parse_bytes_inner(radix: u32, src: &[u8], mut num: i128) -> (i128, u8, Option<&[u8]>) {
    let mut count = 0;
    let mut next = src;
    while let [byte, rest @ ..] = next {
        if let Some(digit) = (*byte as char).to_digit(radix) {
            count += 1;
            num = num.saturating_mul(radix as i128).saturating_add(digit as i128);
        } else if *byte != b'_' || count == 0 {
            return (num, count, Some(next));
        }
        next = rest;
    }
    (num, count, None)
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    pub fn parse_bytes_inner_full() {
        let test = |radix, src: &str, result| {
            assert_eq!(parse_bytes_inner(radix, src.as_bytes(), 0).0, result, "{radix}, {src}")
        };

        test(2, "111", 0b111);
        test(2, "111", 0b111);
        test(8, "177", 0o177);
        test(10, "199", 199);
        test(16, "1ff", 0x1ff);
        test(36, "1_zzz", i128::from_str_radix("1zzz", 36).unwrap());

        test(16, "7fff_ffff_ffff_ffff_ffff_ffff_ffff_fffE", i128::MAX - 1);
        test(16, "7fff_ffff_ffff_ffff_ffff_ffff_ffff_fffF", i128::MAX);
        // must saturate at MAX
        test(16, "Ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff", i128::MAX);
    }

    #[test]
    pub fn parse_bytes_inner_partial() {
        // Can only pass matcher to a macro, as normal variable would be a (useless) binding.
        macro_rules! test {
            ($radix:expr, $src:expr, $num:expr; $result:tt) => {
                assert!(
                    matches!(parse_bytes_inner($radix, $src.as_bytes(), $num), $result),
                    "{}, {}, {}",
                    $radix,
                    $src,
                    $num
                );
            };
        }
        // Assemble floaty: number parsed, digits count, offending rest
        test!(10, "01_234.567_8", 0;
            (1234, 5, Some(b".567_8")));
        // … and feed received 1234 back in, to get whole number & -exp
        test!(10, "567_8", 1234;
            (12345678, 4, None));
    }

    // Convert ParseResult to Decimal Result by impl From
    fn parse_dec(src: &str, exp: i32) -> Result<Decimal, ParseError> {
        let unpacked = super::parse_dec(src, exp)?;
        Ok(Decimal::from_i128_with_scale(unpacked.mantissa, unpacked.scale))
    }

    fn parse_radix_dec(radix: u32, src: &str, exp: i32) -> Result<Decimal, ParseError> {
        let unpacked = super::parse_radix_dec(radix, src, exp)?;
        Ok(Decimal::from_i128_with_scale(unpacked.mantissa, unpacked.scale))
    }

    #[test]
    // cases that don’t have their own Error symbol
    pub fn parse_dec_string() {
        let test = |src, exp, result: &str| {
            if let Err(err) = parse_dec(src, exp) {
                assert_eq!(err.to_string(), result);
            } else {
                panic!("no Err {src}, {exp}")
            }
        };
        test("", 0, "Number is empty, must have an integer part.");
        test(".1", 0, "Number is empty, must have an integer part.");
        test("1.e2", 0, "Fraction empty, consider adding a `0` after the period.");
        test("1abc", 0, "Cannot parse decimal, unexpected \"abc\".");
        test("1 e1", 0, "Cannot parse decimal, unexpected \" e1\".");
        test("1e 1", 0, "Cannot parse decimal, unexpected \" 1\".");
        test("1e+ 1", 0, "Cannot parse decimal, unexpected \" 1\".");
        test("1e- 1", 0, "Cannot parse decimal, unexpected \" 1\".");
        test("1e +1", 0, "Cannot parse decimal, unexpected \" +1\".");
        test("1e -1", 0, "Cannot parse decimal, unexpected \" -1\".");
        test(
            "1e-1",
            50,
            "Invalid exp 49 -- exp must be in the range -28 to 28 inclusive.",
        );
        test(
            "1e60",
            -1,
            "Invalid exp 59 -- exp must be in the range -28 to 28 inclusive.",
        );
        test(
            "1e+80",
            9,
            "Invalid exp 89 -- exp must be in the range -28 to 28 inclusive.",
        );
        test(
            "1",
            99,
            "Invalid exp 99 -- exp must be in the range -28 to 28 inclusive.",
        );
    }

    #[test]
    pub fn parse_dec_other() {
        let test = |src, exp, result| {
            if let Err(err) = parse_dec(src, exp) {
                assert_eq!(err, result, "{src}, {exp}")
            } else {
                panic!("no Err {src}, {exp}")
            }
        };
        test("1e1", -50, ParseError::InvalidExp(-49));
        test("1e-80", 1, ParseError::InvalidExp(-79));
        test("1", -99, ParseError::InvalidExp(-99));
        test("100", 28, ParseError::ExceedsMaximumPossibleValue);
        test("-100", 28, ParseError::LessThanMinimumPossibleValue);
        test(
            "100_000_000_000_000_000_000_000_000_000",
            -1,
            ParseError::ExceedsMaximumPossibleValue,
        );
        test(
            "-100_000_000_000_000_000_000_000_000_000",
            -1,
            ParseError::LessThanMinimumPossibleValue,
        );
        test("9.000_000_000_000_000_000_000_000_000_001", 0, ParseError::Underflow);
    }

    #[test]
    pub fn parse_radix_dec_any() {
        let test = |radix, src, exp, result| {
            if let Err(err) = parse_radix_dec(radix, src, exp) {
                assert_eq!(err, result, "{src}, {exp}")
            } else {
                panic!("no Err {src}, {exp}")
            }
        };
        test(1, "", 0, ParseError::InvalidRadix(1));
        test(37, "", 0, ParseError::InvalidRadix(37));
        test(4, "12_3456", 0, ParseError::Unparseable("456".as_bytes()));
    }
}
