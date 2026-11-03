use core::ops::{Range, RangeInclusive};
use rand_0_10::{
    Rng, RngExt,
    distr::{
        Distribution,
        uniform::{Error, UniformInt, UniformSampler},
    },
};
use rust_decimal::{Decimal, RoundingStrategy};

/// Samples any [`Decimal`] across the full mantissa and scale range.
///
/// ```
/// # use rand_0_10 as rand;
/// # use rand::RngExt;
/// use rust_decimal_rand::rand_0_10::StandardDecimal;
///
/// let value = rand::rng().sample(StandardDecimal);
/// assert!(value.scale() <= 28);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardDecimal;

impl Distribution<Decimal> for StandardDecimal {
    fn sample<R>(&self, rng: &mut R) -> Decimal
    where
        R: Rng + ?Sized,
    {
        Decimal::from_parts(
            rng.next_u32(),
            rng.next_u32(),
            rng.next_u32(),
            rng.random(),
            rng.random_range(0..=Decimal::MAX_SCALE),
        )
    }
}

/// Samples a [`Decimal`] uniformly from a range.
///
/// Samples always have the same scale as the inputs; if the inputs have different scales, the
/// higher scale is used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformDecimal {
    mantissa_sampler: UniformInt<i128>,
    scale: u32,
}

impl UniformDecimal {
    /// Creates a sampler for the range `low..high`.
    ///
    /// # Example
    ///
    /// ```
    /// # use rand_0_10 as rand;
    /// # use rand::RngExt;
    /// # use rust_decimal_macros::dec;
    /// use rust_decimal_rand::rand_0_10::UniformDecimal;
    /// let mut rng = rand::rng();
    /// let random = rng.sample(UniformDecimal::new(dec!(1.00), dec!(2.00)).unwrap());
    /// assert!(random >= dec!(1.00));
    /// assert!(random < dec!(2.00));
    /// assert_eq!(random.scale(), 2);
    /// ```
    pub fn new(low: Decimal, high: Decimal) -> Result<Self, Error> {
        let (low, synced_high) = sync_scales(low, high);
        // `sync_scales` may have rounded `high` down to a coarser scale. If it did, the result is
        // already strictly below the exclusive bound; otherwise step down to the previous value
        // at this scale. If that is not representable then nothing is below `high`, so the
        // range is empty.
        let high = if synced_high < high {
            synced_high
        } else {
            Decimal::try_from_i128_with_scale(synced_high.mantissa() - 1, synced_high.scale())
                .map_err(|_| Error::EmptyRange)?
        };
        Self::new_inclusive(low, high)
    }

    /// Creates a sampler for the range `low..=high`.
    ///
    /// # Example
    ///
    /// ```
    /// # use rand_0_10 as rand;
    /// # use rand::RngExt;
    /// # use rust_decimal_macros::dec;
    /// use rust_decimal_rand::rand_0_10::UniformDecimal;
    /// let mut rng = rand::rng();
    /// let random = rng.sample(UniformDecimal::new_inclusive(dec!(1.00), dec!(2.00)).unwrap());
    /// assert!(random >= dec!(1.00));
    /// assert!(random <= dec!(2.00));
    /// assert_eq!(random.scale(), 2);
    /// ```
    pub fn new_inclusive(low: Decimal, high: Decimal) -> Result<Self, Error> {
        let (low, high) = sync_scales(low, high);

        Ok(Self {
            mantissa_sampler: UniformInt::new_inclusive(low.mantissa(), high.mantissa())?,
            scale: low.scale(),
        })
    }
}

impl Distribution<Decimal> for UniformDecimal {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Decimal {
        let mantissa = self.mantissa_sampler.sample(rng);
        Decimal::from_i128_with_scale(mantissa, self.scale)
    }
}

/// Adds [`Decimal`] sampling methods to every [`Rng`].
///
/// ```
/// # use rand_0_10 as rand;
/// # use rust_decimal_macros::dec;
/// use rust_decimal_rand::rand_0_10::DecimalRng;
///
/// let mut rng = rand::rng();
/// let any = rng.random_decimal();
/// let price = rng.random_decimal_range(dec!(1.00)..dec!(2.00));
/// assert!(price >= dec!(1.00) && price < dec!(2.00));
/// assert_eq!(price.scale(), 2);
/// ```
pub trait DecimalRng: Rng {
    /// Returns any [`Decimal`] across the full mantissa and scale range. See [`StandardDecimal`].
    fn random_decimal(&mut self) -> Decimal {
        StandardDecimal.sample(self)
    }

    /// Returns a [`Decimal`] sampled uniformly from `low..high` or `low..=high`, at the inputs'
    /// scale. See [`UniformDecimal`].
    ///
    /// # Panics
    ///
    /// Panics if the range is empty, as `rand::Rng::random_range` does.
    fn random_decimal_range<T: DecimalRange>(&mut self, range: T) -> Decimal {
        range.uniform().expect("empty decimal range").sample(self)
    }
}

impl<R: Rng + ?Sized> DecimalRng for R {}

/// A range of [`Decimal`]s accepted by [`DecimalRng::random_decimal_range`].
pub trait DecimalRange {
    /// Builds the sampler for this range.
    fn uniform(self) -> Result<UniformDecimal, Error>;
}

impl DecimalRange for Range<Decimal> {
    fn uniform(self) -> Result<UniformDecimal, Error> {
        UniformDecimal::new(self.start, self.end)
    }
}

impl DecimalRange for RangeInclusive<Decimal> {
    fn uniform(self) -> Result<UniformDecimal, Error> {
        let (low, high) = self.into_inner();
        UniformDecimal::new_inclusive(low, high)
    }
}

#[inline]
fn sync_scales(mut low: Decimal, mut high: Decimal) -> (Decimal, Decimal) {
    if low.scale() == high.scale() {
        return (low, high);
    }

    // Set scales to match one another, because we are relying on mantissas'
    // being comparable in order outsource the actual sampling implementation.
    // Padding with zeros is exact, so this does not change either bound.
    let target = low.scale().max(high.scale());
    low.rescale(target);
    high.rescale(target);

    // Edge case: If the values have _wildly_ different scales, the values may not have rescaled far enough to match one another.
    //
    // In this case we drop to the smaller scale. The range must only ever shrink, so `low` is
    // rounded up and `high` is rounded down; rounding to nearest could yield samples outside
    // the requested bounds.
    if low.scale() != high.scale() {
        let scale = low.scale().min(high.scale());
        low = low.round_dp_with_strategy(scale, RoundingStrategy::ToPositiveInfinity);
        high = high.round_dp_with_strategy(scale, RoundingStrategy::ToNegativeInfinity);
    }

    (low, high)
}

#[cfg(test)]
mod rand_tests {
    use rand_0_10::rng;

    use super::*;

    macro_rules! dec {
        ($e:expr) => {
            Decimal::from_str_exact(stringify!($e)).unwrap()
        };
    }

    #[test]
    fn has_random_decimal_instances() {
        let mut rng = rng();
        let random: [Decimal; 32] = core::array::from_fn(|_| rng.random_decimal());
        assert!(random.windows(2).any(|slice| { slice[0] != slice[1] }));
    }

    #[test]
    fn generates_within_range() {
        let mut rng = rng();
        for _ in 0..128 {
            let random = rng.random_decimal_range(dec!(1.00)..dec!(1.05));
            assert!(random < dec!(1.05));
            assert!(random >= dec!(1.00));
        }
    }

    #[test]
    fn generates_within_inclusive_range() {
        let mut rng = rng();
        let mut saw_low = false;
        let mut saw_high = false;
        for _ in 0..256 {
            let random = rng.random_decimal_range(dec!(1.00)..=dec!(1.01));
            assert!(random == dec!(1.00) || random == dec!(1.01));
            if random == dec!(1.00) {
                saw_low = true;
            } else {
                saw_high = true;
            }
        }
        assert!(saw_low && saw_high);
    }

    #[test]
    fn test_edge_case_scales_match() {
        let (low, high) = sync_scales(dec!(1.000_000_000_000_000_000_01), dec!(100_000_000_000_000_000_001));
        assert_eq!(low.scale(), high.scale());
    }

    #[test]
    fn samples_stay_within_bounds_with_mismatched_scales() {
        use core::str::FromStr;
        let mut rng = rng();
        let low = Decimal::from_str("7922816251426433759354395033.4").unwrap();
        let high = Decimal::from_str("7922816251426433759354395034").unwrap();
        // Only 7922816251426433759354395034 lies in [low, high], and nothing lies in [low, high).
        let inclusive = rng.random_decimal_range(low..=high);
        assert!(inclusive >= low && inclusive <= high);
        assert!(UniformDecimal::new(low, high).is_err());
    }

    #[test]
    fn non_empty_range_with_mismatched_scales_is_accepted() {
        use core::str::FromStr;
        let low = Decimal::from_str("-7922816251426433759354395034").unwrap();
        let high = Decimal::from_str("-7922816251426433759354395033.5").unwrap();
        // -7922816251426433759354395034 is the only value in [low, high).
        let uniform = UniformDecimal::new(low, high).unwrap();
        assert_eq!(low, uniform.sample(&mut rng()));
    }

    #[test]
    fn empty_range_at_min_is_an_error_not_a_panic() {
        assert!(UniformDecimal::new(Decimal::MIN, Decimal::MIN).is_err());
    }
}
