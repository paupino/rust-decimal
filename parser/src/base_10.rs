use crate::{DecimalComponents, ParserError, BYTES_TO_OVERFLOW_U64, OVERFLOW_U96, WILL_OVERFLOW_U64};

#[inline]
pub fn parse_radix_10(str: &str) -> Result<DecimalComponents, ParserError> {
    let bytes = str.as_bytes();
    if bytes.len() < BYTES_TO_OVERFLOW_U64 {
        parse_str_radix_10_dispatch::<false, true>(bytes)
    } else {
        parse_str_radix_10_dispatch::<true, true>(bytes)
    }
}

#[inline]
pub fn parse_radix_10_exact(str: &str) -> Result<DecimalComponents, ParserError> {
    let bytes = str.as_bytes();
    if bytes.len() < BYTES_TO_OVERFLOW_U64 {
        parse_str_radix_10_dispatch::<false, false>(bytes)
    } else {
        parse_str_radix_10_dispatch::<true, false>(bytes)
    }
}

#[inline]
fn parse_str_radix_10_dispatch<const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
) -> Result<DecimalComponents, ParserError> {
    match bytes {
        [b, rest @ ..] => byte_dispatch_u64::<false, false, false, BIG, true, ROUND>(rest, 0, 0, *b),
        [] => Err(ParserError::EmptyInput),
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
) -> Result<DecimalComponents, ParserError> {
    if let Some((next, bytes)) = bytes.split_first() {
        byte_dispatch_u64::<POINT, NEG, HAS, BIG, false, ROUND>(bytes, data64, scale, *next)
    } else {
        handle_data::<NEG, HAS>(data64 as u128, scale)
    }
}

/// Dispatch the next non-digit byte:
///
/// * POINT - a decimal point has been seen
/// * NEG - we've encountered a `-` and the number is negative
/// * HAS - a digit has been encountered (when HAS is false it's invalid)
/// * BIG - a number that uses 96 bits instead of only 64 bits
/// * FIRST - true if it is the first byte in the string
/// * ROUND - attempt to round underflow
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
) -> Result<DecimalComponents, ParserError> {
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
) -> Result<DecimalComponents, ParserError> {
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
) -> Result<DecimalComponents, ParserError> {
    // we have already validated that we cannot overflow
    let data64 = data64 * 10 + digit as u64;
    let scale = if POINT { scale + 1 } else { 0 };

    if let Some((next, bytes)) = bytes.split_first() {
        let next = *next;
        if POINT && BIG && scale >= 28 {
            if ROUND {
                maybe_round(data64 as u128, next, scale, POINT, NEG)
            } else {
                Err(ParserError::Underflow)
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
) -> Result<DecimalComponents, ParserError> {
    dispatch_next::<true, NEG, HAS, BIG, ROUND>(bytes, data64, scale)
}

#[inline(never)]
fn handle_separator<const POINT: bool, const NEG: bool, const BIG: bool, const ROUND: bool>(
    bytes: &[u8],
    data64: u64,
    scale: u8,
) -> Result<DecimalComponents, ParserError> {
    dispatch_next::<POINT, NEG, true, BIG, ROUND>(bytes, data64, scale)
}

#[inline(never)]
#[cold]
fn tail_invalid_digit(digit: u8) -> Result<DecimalComponents, ParserError> {
    match digit {
        b'.' => Err(ParserError::MultipleDecimalPoints),
        b'_' => Err(ParserError::InvalidPlaceholderPosition),
        _ => Err(ParserError::InvalidCharacter),
    }
}

#[inline(never)]
#[cold]
fn handle_full_128<const POINT: bool, const NEG: bool, const ROUND: bool>(
    mut data: u128,
    bytes: &[u8],
    scale: u8,
    next_byte: u8,
) -> Result<DecimalComponents, ParserError> {
    let b = next_byte;
    match b {
        b'0'..=b'9' => {
            let digit = u32::from(b - b'0');

            // If the data is going to overflow then we should go into recovery mode
            let next = (data * 10) + digit as u128;
            if overflow_128(next) {
                if !POINT {
                    return Err(ParserError::Overflow);
                }

                if ROUND {
                    maybe_round(data, next_byte, scale, POINT, NEG)
                } else {
                    Err(ParserError::Underflow)
                }
            } else {
                data = next;
                let scale = scale + POINT as u8;
                if let Some((next, bytes)) = bytes.split_first() {
                    let next = *next;
                    if POINT && scale >= 28 {
                        if ROUND {
                            // If it is an underscore at the rounding position we require slightly different handling to look ahead another digit
                            if next == b'_' {
                                if let Some((next, bytes)) = bytes.split_first() {
                                    handle_full_128::<POINT, NEG, ROUND>(data, bytes, scale, *next)
                                } else {
                                    handle_data::<NEG, true>(data, scale)
                                }
                            } else {
                                // Otherwise, we round as usual
                                maybe_round(data, next, scale, POINT, NEG)
                            }
                        } else {
                            Err(ParserError::Underflow)
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
fn maybe_round(
    mut data: u128,
    next_byte: u8,
    mut scale: u8,
    point: bool,
    negative: bool,
) -> Result<DecimalComponents, ParserError> {
    let digit = match next_byte {
        b'0'..=b'9' => u32::from(next_byte - b'0'),
        b'_' => 0, // This is perhaps an error case, but keep this here for compatibility
        b'.' if !point => 0,
        b => return tail_invalid_digit(b),
    };

    // Round at midpoint
    if digit >= 5 {
        data += 1;

        // If the mantissa is now overflowing, round to the next
        // next least significant digit and discard precision
        if overflow_128(data) {
            if scale == 0 {
                return Err(ParserError::OverflowAfterRound);
            }
            data += 4;
            data /= 10;
            scale -= 1;
        }
    }

    if negative {
        handle_data::<true, true>(data, scale)
    } else {
        handle_data::<false, true>(data, scale)
    }
}

#[inline(never)]
fn tail_no_has() -> Result<DecimalComponents, ParserError> {
    return Err(ParserError::NoDigits);
}

#[inline]
fn handle_data<const NEG: bool, const HAS: bool>(data: u128, scale: u8) -> Result<DecimalComponents, ParserError> {
    debug_assert_eq!(data >> 96, 0);
    if !HAS {
        tail_no_has()
    } else {
        Ok(DecimalComponents {
            lo: data as u32,
            mid: (data >> 32) as u32,
            hi: (data >> 64) as u32,
            negative: NEG,
            scale: scale as u32,
        })
    }
}
