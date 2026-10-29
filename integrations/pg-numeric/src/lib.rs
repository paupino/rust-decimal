//! Conversion between [`rust_decimal::Decimal`] and the PostgreSQL `NUMERIC` representation.
//!
//! Shared by `rust_decimal_postgres` and `rust_decimal_diesel`. Most users want one of those
//! crates instead.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use rust_decimal::Decimal;

/// A decimal in PostgreSQL `NUMERIC` form: base-10000 digit groups.
pub struct PostgresDecimal<D> {
    pub neg: bool,
    pub weight: i16,
    pub scale: u16,
    pub digits: D,
}

/// Converts a PostgreSQL `NUMERIC` to a [`Decimal`], rounding to at most 28 decimal places.
///
/// Returns `None` if the value is out of range for a [`Decimal`].
pub fn from_postgres<D: ExactSizeIterator<Item = u16>>(
    PostgresDecimal {
        neg,
        scale,
        digits,
        weight,
    }: PostgresDecimal<D>,
) -> Option<Decimal> {
    let digits = digits.into_iter().collect::<Vec<_>>();

    let weight = weight as i32;
    let fractionals_part_count = digits.len() as i32 + (-weight) - 1;
    let integers_part_count = weight + 1;

    let mut integer_part = Decimal::ZERO;
    if integers_part_count > 0 {
        let (start_integers, integer_digits_count) = if integers_part_count > digits.len() as i32 {
            (integers_part_count - digits.len() as i32, digits.len() as i32)
        } else {
            (0, integers_part_count)
        };
        for &digit in digits.iter().take(integer_digits_count as usize) {
            integer_part = integer_part.checked_mul(Decimal::from_i128_with_scale(10i128.pow(4), 0))?;
            integer_part = integer_part.checked_add(Decimal::new(digit as i64, 0))?;
        }
        let scale_pow = 10i128.checked_pow(4 * start_integers as u32)?;
        integer_part = integer_part.checked_mul(Decimal::try_from_i128_with_scale(scale_pow, 0).ok()?)?;
    }
    if fractionals_part_count <= 0 {
        integer_part.set_sign_negative(neg);
        integer_part.rescale((scale as u32).min(Decimal::MAX_SCALE));
        return Some(integer_part);
    }

    let Some((first_group_index, first_group)) = digits.iter().enumerate().find(|(_, digit)| **digit != 0) else {
        integer_part.set_sign_negative(neg);
        integer_part.rescale((scale as u32).min(Decimal::MAX_SCALE));
        return Some(integer_part);
    };

    let mut first_group_digits = 1i128;
    let mut first_group_divisor = 10u16;
    while *first_group >= first_group_divisor {
        first_group_digits += 1;
        first_group_divisor *= 10;
    }
    let exponent = 4 * (weight as i128 - first_group_index as i128) + first_group_digits - 1;
    let mut prefix = 0u128;
    let mut prefix_digits = 0usize;
    let mut sticky = false;
    for (group_index, &group) in digits.iter().enumerate().skip(first_group_index) {
        let group_digits = if group_index == first_group_index {
            first_group_digits
        } else {
            4
        };
        let mut divisor = 10u16.pow((group_digits - 1) as u32);
        for _ in 0..group_digits {
            let digit = (group / divisor) % 10;
            if prefix_digits < 30 {
                prefix = prefix * 10 + digit as u128;
                prefix_digits += 1;
            } else {
                sticky |= digit != 0;
            }
            divisor /= 10;
        }
    }

    let max_mantissa = (1u128 << 96) - 1;
    let mut rounded = None;
    for candidate_scale in (0..=Decimal::MAX_SCALE).rev() {
        let coefficient_digits = exponent + candidate_scale as i128 + 1;
        if coefficient_digits > 29 {
            continue;
        }

        let (coefficient, guard, remaining_nonzero) = if coefficient_digits < 0 {
            (0, 0, false)
        } else if coefficient_digits == 0 {
            let divisor = 10u128.pow((prefix_digits - 1) as u32);
            (0, (prefix / divisor) as u8, prefix % divisor != 0 || sticky)
        } else {
            let coefficient_digits = coefficient_digits as usize;
            if coefficient_digits >= prefix_digits {
                (
                    prefix * 10u128.pow((coefficient_digits - prefix_digits) as u32),
                    0,
                    sticky,
                )
            } else {
                let divisor = 10u128.pow((prefix_digits - coefficient_digits) as u32);
                let guard_divisor = divisor / 10;
                (
                    prefix / divisor,
                    ((prefix / guard_divisor) % 10) as u8,
                    prefix % guard_divisor != 0 || sticky,
                )
            }
        };

        let increment = guard > 5
            || (guard == 5 && (remaining_nonzero || candidate_scale == Decimal::MAX_SCALE || coefficient % 2 == 1));
        let candidate = coefficient + u128::from(increment);
        if candidate <= max_mantissa {
            rounded = Some((candidate, candidate_scale));
            break;
        }
    }

    let (coefficient, candidate_scale) = rounded?;
    let mut result = Decimal::try_from_i128_with_scale(coefficient as i128, candidate_scale).ok()?;
    result.set_sign_negative(neg);
    result.rescale((scale as u32).min(Decimal::MAX_SCALE));
    Some(result)
}

/// Converts a [`Decimal`] to PostgreSQL `NUMERIC` form.
pub fn to_postgres(value: Decimal) -> PostgresDecimal<Vec<i16>> {
    if value.is_zero() {
        return PostgresDecimal {
            neg: false,
            weight: 0,
            scale: value.scale() as u16,
            digits: vec![],
        };
    }
    let scale = value.scale() as u16;

    let groups_diff = scale & 0x3; // groups_diff = scale % 4

    // At most 96 bits; scaling by up to 10^3 still fits in a u128.
    let mut mantissa = value.mantissa().unsigned_abs();

    if groups_diff > 0 {
        let remainder = 4 - groups_diff;
        mantissa *= 10u128.pow(u32::from(remainder));
    }

    // array to store max mantissa of Decimal in Postgres decimal format
    const MAX_GROUP_COUNT: usize = 8;
    let mut digits = Vec::with_capacity(MAX_GROUP_COUNT);

    while mantissa != 0 {
        digits.push((mantissa % 10000) as i16);
        mantissa /= 10000;
    }
    digits.reverse();
    let digits_after_decimal = scale.div_ceil(4);
    let weight = digits.len() as i16 - digits_after_decimal as i16 - 1;

    let unnecessary_zeroes = if weight >= 0 {
        let index_of_decimal = (weight + 1) as usize;
        digits
            .get(index_of_decimal..)
            .expect("enough digits exist")
            .iter()
            .rev()
            .take_while(|i| **i == 0)
            .count()
    } else {
        0
    };
    let relevant_digits = digits.len() - unnecessary_zeroes;
    digits.truncate(relevant_digits);

    PostgresDecimal {
        neg: value.is_sign_negative(),
        digits,
        scale,
        weight,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn weight_i16_min_does_not_panic() {
        // -weight on i16::MIN overflows in debug builds; value is too small for Decimal
        let result = from_postgres(PostgresDecimal {
            neg: false,
            weight: i16::MIN,
            scale: 0,
            digits: core::iter::once(1u16),
        });
        assert!(result.is_none() || result == Some(Decimal::ZERO));
    }

    #[test]
    fn large_weight_returns_none_not_panic() {
        // weight=11 → start_integers=11 → 10i128.pow(44) overflows; must return None, not panic.
        // Represents e.g. 1e44 stored as ndigits=1, weight=11, digits=[1].
        let result = from_postgres(PostgresDecimal {
            neg: false,
            weight: 11,
            scale: 0,
            digits: core::iter::once(1u16),
        });
        assert!(
            result.is_none(),
            "expected None for value exceeding Decimal::MAX, got {result:?}"
        );
    }
}
