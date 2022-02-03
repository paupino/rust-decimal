use crate::{
    constants::{MAX_PRECISION, MAX_STR_BUFFER_SIZE},
    ops::array::{div_by_u32, is_all_zero},
    Decimal,
};

use arrayvec::{ArrayString, ArrayVec};

// impl that doesn't allocate for serialization purposes.
pub(crate) fn to_str_internal(
    value: &Decimal,
    append_sign: bool,
    precision: Option<usize>,
) -> (ArrayString<MAX_STR_BUFFER_SIZE>, Option<usize>) {
    // Get the scale - where we need to put the decimal point
    let scale = value.scale() as usize;

    // Convert to a string and manipulate that (neg at front, inject decimal)
    let mut chars = ArrayVec::<_, MAX_STR_BUFFER_SIZE>::new();
    let mut working = value.mantissa_array3();
    while !is_all_zero(&working) {
        let remainder = div_by_u32(&mut working, 10u32);
        chars.push(char::from(b'0' + remainder as u8));
    }
    while scale > chars.len() {
        chars.push('0');
    }

    let (prec, additional) = match precision {
        Some(prec) => {
            let max: usize = MAX_PRECISION.into();
            if prec > max {
                (max, Some(prec - max))
            } else {
                (prec, None)
            }
        }
        None => (scale, None),
    };

    let len = chars.len();
    let whole_len = len - scale;
    let mut rep = ArrayString::new();
    // Append the negative sign if necessary while also keeping track of the length of an "empty" string representation
    let empty_len = if append_sign && value.is_sign_negative() {
        rep.push('-');
        1
    } else {
        0
    };
    for i in 0..whole_len + prec {
        if i == len - scale {
            if i == 0 {
                rep.push('0');
            }
            rep.push('.');
        }

        if i >= len {
            rep.push('0');
        } else {
            let c = chars[len - i - 1];
            rep.push(c);
        }
    }

    // corner case for when we truncated everything in a low fractional
    if rep.len() == empty_len {
        rep.push('0');
    }

    (rep, additional)
}

#[cfg(test)]
mod test {
    use crate::Decimal;
    use arrayvec::ArrayString;
    use core::{fmt::Write, str::FromStr};

    #[test]
    fn display_does_not_overflow_max_capacity() {
        let num = Decimal::from_str("1.2").unwrap();
        let mut buffer = ArrayString::<64>::new();
        let _ = buffer.write_fmt(format_args!("{:.31}", num)).unwrap();
        assert_eq!("1.2000000000000000000000000000000", buffer.as_str());
    }
}
