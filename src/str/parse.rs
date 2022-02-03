use crate::{
    constants::{BYTES_TO_OVERFLOW_U64, OVERFLOW_U96, WILL_OVERFLOW_U64},
    error::{tail_error, Error},
    Decimal,
};

// dedicated implementation for the most common case.
#[inline]
pub(crate) fn parse_str_radix_10(str: &str) -> Result<Decimal, crate::Error> {
    let bytes = str.as_bytes();
    if bytes.len() < BYTES_TO_OVERFLOW_U64 {
        parse_str_radix_10_dispatch::<false, true>(bytes)
    } else {
        parse_str_radix_10_dispatch::<true, true>(bytes)
    }
}

#[inline]
pub(crate) fn parse_str_radix_10_exact(str: &str) -> Result<Decimal, crate::Error> {
    let bytes = str.as_bytes();
    if bytes.len() < BYTES_TO_OVERFLOW_U64 {
        parse_str_radix_10_dispatch::<false, false>(bytes)
    } else {
        parse_str_radix_10_dispatch::<true, false>(bytes)
    }
}

#[inline]
fn parse_str_radix_10_dispatch<const BIG: bool, const ROUND: bool>(bytes: &[u8]) -> Result<Decimal, crate::Error> {
    match bytes {
        [b, rest @ ..] => byte_dispatch_u64::<false, false, false, BIG, true, ROUND>(rest, 0, 0, *b),
        [] => tail_error("Invalid decimal: empty"),
    }
}

#[inline]
fn overflow_64(val: u64) -> bool {
    val >= WILL_OVERFLOW_U64
}

#[inline]
fn overflow_128(val: u128) -> bool {
    val >= OVERFLOW_U96
}

/// Dispatch the next byte:
///
/// * POINT - a decimal point has been seen
/// * NEG - we've encountered a `-` and the number is negative
/// * HAS - a digit has been encountered (when HAS is false it's invalid)
/// * BIG - a number that uses 96 bits instead of only 64 bits
/// * FIRST - true if it is the first byte in the string
#[inline]
fn dispatch_next<const POINT: bool, const NEG: bool, const HAS: bool, const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
) -> Result<Decimal, crate::Error> {
    if let Some((next, bytes)) = bytes.split_first() {
        byte_dispatch_u64::<POINT, NEG, HAS, BIG, false, ROUND>(bytes, data64, scale, *next)
    } else {
        handle_data::<NEG, HAS>(data64 as u128, scale)
    }
}

#[inline(never)]
fn non_digit_dispatch_u64<
    const POINT: bool,
    const NEG: bool,
    const HAS: bool,
    const BIG: bool,
    const FIRST: bool,
    const ROUND: bool,
>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
    b: u8,
) -> Result<Decimal, crate::Error> {
    match b {
        b'-' if FIRST && !HAS => dispatch_next::<false, true, false, BIG, ROUND>(bytes, data64, scale),
        b'+' if FIRST && !HAS => dispatch_next::<false, false, false, BIG, ROUND>(bytes, data64, scale),
        b'_' if HAS => handle_separator::<POINT, NEG, BIG, ROUND>(bytes, data64, scale),
        b => tail_invalid_digit(b),
    }
}

#[inline]
fn byte_dispatch_u64<
    const POINT: bool,
    const NEG: bool,
    const HAS: bool,
    const BIG: bool,
    const FIRST: bool,
    const ROUND: bool,
>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
    b: u8,
) -> Result<Decimal, crate::Error> {
    match b {
        b'0'..=b'9' => handle_digit_64::<POINT, NEG, BIG, ROUND>(bytes, data64, scale, b - b'0'),
        b'.' if !POINT => handle_point::<NEG, HAS, BIG, ROUND>(bytes, data64, scale),
        b => non_digit_dispatch_u64::<POINT, NEG, HAS, BIG, FIRST, ROUND>(bytes, data64, scale, b),
    }
}

#[inline(never)]
fn handle_digit_64<const POINT: bool, const NEG: bool, const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
    digit: u8,
) -> Result<Decimal, crate::Error> {
    // we have already validated that we cannot overflow
    let data64 = data64 * 10 + digit as u64;
    let scale = if POINT { scale + 1 } else { 0 };

    if let Some((next, bytes)) = bytes.split_first() {
        let next = *next;
        if POINT && BIG && scale >= 28 {
            if ROUND {
                maybe_round(data64 as u128, next, scale, POINT, NEG)
            } else {
                Err(crate::Error::Underflow)
            }
        } else if BIG && overflow_64(data64) {
            handle_full_128::<POINT, NEG, ROUND>(data64 as u128, bytes, scale, next)
        } else {
            byte_dispatch_u64::<POINT, NEG, true, BIG, false, ROUND>(bytes, data64, scale, next)
        }
    } else {
        let data: u128 = data64 as u128;

        handle_data::<NEG, true>(data, scale)
    }
}

#[inline(never)]
fn handle_point<const NEG: bool, const HAS: bool, const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
) -> Result<Decimal, crate::Error> {
    dispatch_next::<true, NEG, HAS, BIG, ROUND>(bytes, data64, scale)
}

#[inline(never)]
fn handle_separator<const POINT: bool, const NEG: bool, const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
) -> Result<Decimal, crate::Error> {
    dispatch_next::<POINT, NEG, true, BIG, ROUND>(bytes, data64, scale)
}

#[inline(never)]
#[cold]
fn tail_invalid_digit(digit: u8) -> Result<Decimal, crate::Error> {
    match digit {
        b'.' => tail_error("Invalid decimal: two decimal points"),
        b'_' => tail_error("Invalid decimal: must start lead with a number"),
        _ => tail_error("Invalid decimal: unknown character"),
    }
}

#[inline(never)]
#[cold]
fn handle_full_128<const POINT: bool, const NEG: bool, const ROUND: bool>(
    mut data: u128,
    bytes: &[u8],
    scale: u8,
    next_byte: u8,
) -> Result<Decimal, crate::Error> {
    let b = next_byte;
    match b {
        b'0'..=b'9' => {
            let digit = u32::from(b - b'0');

            // If the data is going to overflow then we should go into recovery mode
            let next = (data * 10) + digit as u128;
            if overflow_128(next) {
                if !POINT {
                    return tail_error("Invalid decimal: overflow from too many digits");
                }

                if ROUND {
                    if digit >= 5 {
                        data += 1;
                    }
                } else {
                    return Err(Error::Underflow);
                }
                handle_data::<NEG, true>(data, scale)
            } else {
                data = next;
                let scale = scale + POINT as u8;
                if let Some((next, bytes)) = bytes.split_first() {
                    let next = *next;
                    if POINT && scale >= 28 {
                        if ROUND {
                            maybe_round(data, next, scale, POINT, NEG)
                        } else {
                            Err(crate::Error::Underflow)
                        }
                    } else {
                        handle_full_128::<POINT, NEG, ROUND>(data, bytes, scale, next)
                    }
                } else {
                    handle_data::<NEG, true>(data, scale)
                }
            }
        }
        b'.' if !POINT => {
            // This call won't tail?
            if let Some((next, bytes)) = bytes.split_first() {
                handle_full_128::<true, NEG, ROUND>(data, bytes, scale, *next)
            } else {
                handle_data::<NEG, true>(data, scale)
            }
        }
        b'_' => {
            if let Some((next, bytes)) = bytes.split_first() {
                handle_full_128::<POINT, NEG, ROUND>(data, bytes, scale, *next)
            } else {
                handle_data::<NEG, true>(data, scale)
            }
        }
        b => tail_invalid_digit(b),
    }
}

#[inline(never)]
#[cold]
fn maybe_round(mut data: u128, next_byte: u8, scale: u8, point: bool, negative: bool) -> Result<Decimal, crate::Error> {
    let digit = match next_byte {
        b'0'..=b'9' => u32::from(next_byte - b'0'),
        b'_' => 0, // this should be an invalid string?
        b'.' if point => 0,
        b => return tail_invalid_digit(b),
    };

    // Round at midpoint
    if digit >= 5 {
        data += 1;
        if overflow_128(data) {
            // Highly unlikely scenario which is more indicative of a bug
            return tail_error("Invalid decimal: overflow when rounding");
        }
    }

    if negative {
        handle_data::<true, true>(data, scale)
    } else {
        handle_data::<false, true>(data, scale)
    }
}

#[inline(never)]
fn tail_no_has() -> Result<Decimal, crate::Error> {
    tail_error("Invalid decimal: no digits found")
}

#[inline]
fn handle_data<const NEG: bool, const HAS: bool>(data: u128, scale: u8) -> Result<Decimal, crate::Error> {
    debug_assert_eq!(data >> 96, 0);
    if !HAS {
        tail_no_has()
    } else {
        Ok(Decimal::from_parts(
            data as u32,
            (data >> 32) as u32,
            (data >> 64) as u32,
            NEG,
            scale as u32,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Decimal;

    #[test]
    fn from_str_rounding_0() {
        assert_eq!(
            parse_str_radix_10("1.234").unwrap().unpack(),
            Decimal::new(1234, 3).unpack()
        );
    }

    #[test]
    fn from_str_rounding_1() {
        assert_eq!(
            parse_str_radix_10("11111_11111_11111.11111_11111_11111")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(11_111_111_111_111_111_111_111_111_111, 14).unpack()
        );
    }

    #[test]
    fn from_str_rounding_2() {
        assert_eq!(
            parse_str_radix_10("11111_11111_11111.11111_11111_11115")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(11_111_111_111_111_111_111_111_111_112, 14).unpack()
        );
    }

    #[test]
    fn from_str_rounding_3() {
        assert_eq!(
            parse_str_radix_10("11111_11111_11111.11111_11111_11195")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(1_111_111_111_111_111_111_111_111_1120, 14).unpack() // was Decimal::from_i128_with_scale(1_111_111_111_111_111_111_111_111_112, 13)
        );
    }

    #[test]
    fn from_str_rounding_4() {
        assert_eq!(
            parse_str_radix_10("99999_99999_99999.99999_99999_99995")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(10_000_000_000_000_000_000_000_000_000, 13).unpack() // was Decimal::from_i128_with_scale(1_000_000_000_000_000_000_000_000_000, 12)
        );
    }

    #[test]
    fn from_str_no_rounding_0() {
        assert_eq!(
            parse_str_radix_10_exact("1.234").unwrap().unpack(),
            Decimal::new(1234, 3).unpack()
        );
    }

    #[test]
    fn from_str_no_rounding_1() {
        assert_eq!(
            parse_str_radix_10_exact("11111_11111_11111.11111_11111_11111"),
            Err(Error::Underflow)
        );
    }

    #[test]
    fn from_str_no_rounding_2() {
        assert_eq!(
            parse_str_radix_10_exact("11111_11111_11111.11111_11111_11115"),
            Err(Error::Underflow)
        );
    }

    #[test]
    fn from_str_no_rounding_3() {
        assert_eq!(
            parse_str_radix_10_exact("11111_11111_11111.11111_11111_11195"),
            Err(Error::Underflow)
        );
    }

    #[test]
    fn from_str_no_rounding_4() {
        assert_eq!(
            parse_str_radix_10_exact("99999_99999_99999.99999_99999_99995"),
            Err(Error::Underflow)
        );
    }

    #[test]
    fn from_str_many_pointless_chars() {
        assert_eq!(
            parse_str_radix_10("00________________________________________________________________001.1")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(11, 1).unpack()
        );
    }

    #[test]
    fn from_str_leading_0s_1() {
        assert_eq!(
            parse_str_radix_10("00001.1").unwrap().unpack(),
            Decimal::from_i128_with_scale(11, 1).unpack()
        );
    }

    #[test]
    fn from_str_leading_0s_2() {
        assert_eq!(
            parse_str_radix_10("00000_00000_00000_00000_00001.00001")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(100001, 5).unpack()
        );
    }

    #[test]
    fn from_str_leading_0s_3() {
        assert_eq!(
            parse_str_radix_10("0.00000_00000_00000_00000_00000_00100")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(1, 28).unpack()
        );
    }

    #[test]
    fn from_str_trailing_0s_1() {
        assert_eq!(
            parse_str_radix_10("0.00001_00000_00000").unwrap().unpack(),
            Decimal::from_i128_with_scale(10_000_000_000, 15).unpack()
        );
    }

    #[test]
    fn from_str_trailing_0s_2() {
        assert_eq!(
            parse_str_radix_10("0.00001_00000_00000_00000_00000_00000")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(100_000_000_000_000_000_000_000, 28).unpack()
        );
    }

    #[test]
    fn from_str_overflow_1() {
        assert_eq!(
            parse_str_radix_10("99999_99999_99999_99999_99999_99999.99999"),
            // The original implementation returned
            //              Ok(10000_00000_00000_00000_00000_0000)
            // Which is a bug!
            Err(Error::from("Invalid decimal: overflow from too many digits"))
        );
    }

    #[test]
    fn from_str_overflow_2() {
        assert!(
            parse_str_radix_10("99999_99999_99999_99999_99999_11111.11111").is_err(),
            // The original implementation is 'overflow from scale mismatch'
            // but we got rid of that now
        );
    }

    #[test]
    fn from_str_overflow_3() {
        assert!(
            parse_str_radix_10("99999_99999_99999_99999_99999_99994").is_err() // We could not get into 'overflow when rounding' or 'overflow from carry'
                                                                               // in the original implementation because the rounding logic before prevented it
        );
    }

    #[test]
    fn from_str_overflow_4() {
        assert_eq!(
            // This does not overflow, moving the decimal point 1 more step would result in
            // 'overflow from too many digits'
            parse_str_radix_10("99999_99999_99999_99999_99999_999.99")
                .unwrap()
                .unpack(),
            Decimal::from_i128_with_scale(10_000_000_000_000_000_000_000_000_000, 0).unpack()
        );
    }

    #[test]
    fn from_str_edge_cases_1() {
        assert_eq!(parse_str_radix_10(""), Err(Error::from("Invalid decimal: empty")));
    }

    #[test]
    fn from_str_edge_cases_2() {
        assert_eq!(
            parse_str_radix_10("0.1."),
            Err(Error::from("Invalid decimal: two decimal points"))
        );
    }

    #[test]
    fn from_str_edge_cases_3() {
        assert_eq!(
            parse_str_radix_10("_"),
            Err(Error::from("Invalid decimal: must start lead with a number"))
        );
    }

    #[test]
    fn from_str_edge_cases_4() {
        assert_eq!(
            parse_str_radix_10("1?2"),
            Err(Error::from("Invalid decimal: unknown character"))
        );
    }

    #[test]
    fn from_str_edge_cases_5() {
        assert_eq!(
            parse_str_radix_10("."),
            Err(Error::from("Invalid decimal: no digits found"))
        );
    }
}
