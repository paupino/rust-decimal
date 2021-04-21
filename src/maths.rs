use crate::prelude::*;

const TWO: Decimal = Decimal::from_parts_raw(2, 0, 0, 0);
const PI: Decimal = Decimal::from_parts_raw(1102470953, 185874565, 1703060790, 1835008);
const LN2: Decimal = Decimal::from_parts_raw(2831677809, 328455696, 3757558395, 1900544);
const EXP_TOLERANCE: Decimal = Decimal::from_parts(2, 0, 0, false, 7);

/// Trait exposing various mathematical operations that can be applied using a Decimal. This is only
/// present when the `maths` feature has been enabled.
pub trait MathematicalOps {
    /// The estimated exponential function, e<sup>x</sup>, rounded to 8 decimal places. Stops
    /// calculating when it is within tolerance of roughly 0.000002 in order to prevent
    /// multiplication overflow.
    fn exp(&self) -> Decimal;

    /// The estimated exponential function, e<sup>x</sup>, rounded to 8 decimal places. Stops
    /// calculating when it is within `tolerance`.
    /// Multiplication overflows are likely if you are not careful with the size of `tolerance`.
    /// It is recommended to set the `tolerance` larger for larger numbers and smaller for smaller
    /// numbers to avoid multiplication overflow.
    fn exp_with_tolerance(&self, tolerance: Decimal) -> Decimal;

    /// Raise self to the given unsigned integer exponent: x<sup>y</sup>
    fn powi(&self, exp: u64) -> Decimal;

    /// Raise self to the given unsigned integer exponent x<sup>y</sup> returning
    /// `None` on overflow.
    fn checked_powi(&self, exp: u64) -> Option<Decimal>;

    /// The square root of a Decimal. Uses a standard Babylonian method.
    fn sqrt(&self) -> Option<Decimal>;

    /// The natural logarithm for a Decimal. Uses a [fast estimation algorithm](https://en.wikipedia.org/wiki/Natural_logarithm#High_precision)
    /// This is more accurate on larger numbers and less on numbers less than 1.
    fn ln(&self) -> Decimal;

    /// Abramowitz Approximation of Error Function from [wikipedia](https://en.wikipedia.org/wiki/Error_function#Numerical_approximations)
    fn erf(&self) -> Decimal;

    /// The Cumulative distribution function for a Normal distribution
    fn norm_cdf(&self) -> Decimal;

    /// The Probability density function for a Normal distribution
    fn norm_pdf(&self) -> Decimal;
}

impl MathematicalOps for Decimal {
    /// The estimated exponential function, e<sup>x</sup>, rounded to 8 decimal places. Stops
    /// calculating when it is within tolerance of roughly 0.000002 in order to prevent
    /// multiplication overflow.
    fn exp(&self) -> Decimal {
        self.exp_with_tolerance(EXP_TOLERANCE)
    }

    /// The estimated exponential function, e<sup>x</sup>, rounded to 8 decimal places. Stops
    /// calculating when it is within `tolerance`.
    /// Multiplication overflows are likely if you are not careful with the size of `tolerance`.
    /// It is recommended to set the `tolerance` larger for larger numbers and smaller for smaller
    /// numbers to avoid multiplication overflow.
    #[inline]
    fn exp_with_tolerance(&self, tolerance: Decimal) -> Decimal {
        if self == &Decimal::ZERO {
            return Decimal::ONE;
        }

        let mut term = *self;
        let mut result = self + Decimal::one();
        let mut prev_result: Option<Decimal> = None;
        let mut factorial = Decimal::one();
        let mut n = TWO;
        let twenty_four = Decimal::new(24, 0);

        // Needs rounding because multiplication overflows otherwise.
        while (prev_result.is_none() || (result - prev_result.unwrap()).abs() > tolerance) && n < twenty_four {
            prev_result = Some(result);
            term = self * term.round_dp(8);
            factorial *= n;
            result += (term / factorial).round_dp(8);
            n += Decimal::one();
        }

        result
    }

    /// Raise self to the given unsigned integer exponent: x<sup>y</sup>
    fn powi(&self, exp: u64) -> Decimal {
        match self.checked_powi(exp) {
            Some(result) => result,
            None => panic!("Pow overflowed"),
        }
    }

    fn checked_powi(&self, exp: u64) -> Option<Decimal> {
        match exp {
            0 => Some(Decimal::one()),
            1 => Some(*self),
            2 => self.checked_mul(*self),
            _ => {
                // Get the squared value
                let squared = match self.checked_mul(*self) {
                    Some(s) => s,
                    None => return None,
                };
                // Square self once and make an infinite sized iterator of the square.
                let iter = core::iter::repeat(squared);

                // We then take half of the exponent to create a finite iterator and then multiply those together.
                let mut product = Decimal::one();
                for x in iter.take((exp / 2) as usize) {
                    match product.checked_mul(x) {
                        Some(r) => product = r,
                        None => return None,
                    };
                }

                // If the exponent is odd we still need to multiply once more
                if exp % 2 > 0 {
                    self.checked_mul(product)
                } else {
                    Some(product)
                }
            }
        }
    }

    /// The square root of a Decimal. Uses a standard Babylonian method.
    fn sqrt(&self) -> Option<Decimal> {
        if self.is_sign_negative() {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        // Start with an arbitrary number as the first guess
        let mut result = self / TWO;
        let mut last = result + Decimal::one();

        // Keep going while the difference is larger than the tolerance
        let mut circuit_breaker = 0;
        while last != result {
            circuit_breaker += 1;
            assert!(circuit_breaker < 1000, "geo mean circuit breaker");

            last = result;
            result = (result + self / result) / TWO;
        }

        Some(result)
    }

    /// The natural logarithm for a Decimal. Uses a [fast estimation algorithm](https://en.wikipedia.org/wiki/Natural_logarithm#High_precision)
    /// This is more accurate on larger numbers and less on numbers less than 1.
    fn ln(&self) -> Decimal {
        if self.is_sign_positive() {
            if self == &Decimal::ONE {
                Decimal::ZERO
            } else {
                let s = self * Decimal::new(256, 0);
                let arith_geo_mean = arithmetic_geo_mean_of_2(&Decimal::one(), &(Decimal::new(4, 0) / s));

                PI / (arith_geo_mean * TWO) - (Decimal::new(8, 0) * LN2)
            }
        } else {
            Decimal::ZERO
        }
    }

    /// Abramowitz Approximation of Error Function from [wikipedia](https://en.wikipedia.org/wiki/Error_function#Numerical_approximations)
    fn erf(&self) -> Decimal {
        if self.is_sign_positive() {
            let one = &Decimal::one();

            let xa1 = self * Decimal::from_str("0.0705230784").unwrap();
            let xa2 = self.powi(2) * Decimal::from_str("0.0422820123").unwrap();
            let xa3 = self.powi(3) * Decimal::from_str("0.0092705272").unwrap();
            let xa4 = self.powi(4) * Decimal::from_str("0.0001520143").unwrap();
            let xa5 = self.powi(5) * Decimal::from_str("0.0002765672").unwrap();
            let xa6 = self.powi(6) * Decimal::from_str("0.0000430638").unwrap();

            let sum = one + xa1 + xa2 + xa3 + xa4 + xa5 + xa6;
            one - (one / sum.powi(16))
        } else {
            -self.abs().erf()
        }
    }

    /// The Cumulative distribution function for a Normal distribution
    fn norm_cdf(&self) -> Decimal {
        (Decimal::one() + (self / Decimal::from_str("1.4142135623730951").unwrap()).erf()) / TWO
    }

    /// The Probability density function for a Normal distribution
    fn norm_pdf(&self) -> Decimal {
        let sqrt2pi = Decimal::from_parts_raw(2133383024, 2079885984, 1358845910, 1835008);
        (-self.powi(2) / TWO).exp() / sqrt2pi
    }
}

/// Returns the convergence of both the arithmetic and geometric mean.
/// Used internally.
fn arithmetic_geo_mean_of_2(a: &Decimal, b: &Decimal) -> Decimal {
    const TOLERANCE: Decimal = Decimal::from_parts(5, 0, 0, false, 7);
    let diff = (a - b).abs();

    if diff < TOLERANCE {
        *a
    } else {
        arithmetic_geo_mean_of_2(&mean_of_2(a, b), &geo_mean_of_2(a, b))
    }
}

/// The Arithmetic mean. Used internally.
fn mean_of_2(a: &Decimal, b: &Decimal) -> Decimal {
    (a + b) / TWO
}

/// The geometric mean. Used internally.
fn geo_mean_of_2(a: &Decimal, b: &Decimal) -> Decimal {
    (a * b).sqrt().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn test_geo_mean_of_2() {
        let test_cases = &[
            (
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("2").unwrap(),
            ),
            (
                Decimal::from_str("4").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("3.4641016151377545870548926830").unwrap(),
            ),
            (
                Decimal::from_str("12").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("6.000000000000000000000000000").unwrap(),
            ),
        ];

        for case in test_cases {
            assert_eq!(case.2, geo_mean_of_2(&case.0, &case.1));
        }
    }

    #[test]
    fn test_mean_of_2() {
        let test_cases = &[
            (
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("2").unwrap(),
            ),
            (
                Decimal::from_str("4").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("3.5").unwrap(),
            ),
            (
                Decimal::from_str("12").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("7.5").unwrap(),
            ),
        ];

        for case in test_cases {
            assert_eq!(case.2, mean_of_2(&case.0, &case.1));
        }
    }
}
