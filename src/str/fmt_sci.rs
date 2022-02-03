use crate::{
    ops::array::{div_by_u32, is_all_zero},
    Decimal,
};

use core::fmt;

pub(crate) fn fmt_scientific_notation(
    value: &Decimal,
    exponent_symbol: &str,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    // Get the scale - this is the e value. With multiples of 10 this may get bigger.
    let mut exponent = -(value.scale() as isize);

    // Convert the integral to a string
    let mut chars = Vec::new();
    let mut working = value.mantissa_array3();
    while !is_all_zero(&working) {
        let remainder = div_by_u32(&mut working, 10u32);
        chars.push(char::from(b'0' + remainder as u8));
    }

    // First of all, apply scientific notation rules. That is:
    //  1. If non-zero digit comes first, move decimal point left so that e is a positive integer
    //  2. If decimal point comes first, move decimal point right until after the first non-zero digit
    // Since decimal notation naturally lends itself this way, we just need to inject the decimal
    // point in the right place and adjust the exponent accordingly.

    let len = chars.len();
    let mut rep;
    if len > 1 {
        if chars.iter().take(len - 1).all(|c| *c == '0') {
            // Chomp off the zero's.
            rep = chars.iter().skip(len - 1).collect::<String>();
        } else {
            chars.insert(len - 1, '.');
            rep = chars.iter().rev().collect::<String>();
        }
        exponent += (len - 1) as isize;
    } else {
        rep = chars.iter().collect::<String>();
    }

    rep.push_str(exponent_symbol);
    rep.push_str(&exponent.to_string());
    f.pad_integral(value.is_sign_positive(), "", &rep)
}
