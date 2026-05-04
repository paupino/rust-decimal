use crate::Decimal;
use rand_0_10::{
    Rng, RngExt,
    distr::{
        Distribution, StandardUniform,
        uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler},
    },
};

impl Distribution<Decimal> for StandardUniform {
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

impl SampleUniform for Decimal {
    type Sampler = DecimalSampler;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecimalSampler {
    mantissa_sampler: UniformInt<i128>,
    scale: u32,
}

impl UniformSampler for DecimalSampler {
    type X = Decimal;

    /// Creates a new sampler that will yield random decimal objects between `low` and `high`.
    ///
    /// The sampler will always provide decimals at the same scale as the inputs; if the inputs
    /// have different scales, the higher scale is used.
    ///
    /// # Example
    ///
    /// ```
    /// # use rand_0_10 as rand;
    /// # use rand::RngExt;
    /// # use rust_decimal_macros::dec;
    /// let mut rng = rand::rng();
    /// let random = rng.random_range(dec!(1.00)..dec!(2.00));
    /// assert!(random >= dec!(1.00));
    /// assert!(random < dec!(2.00));
    /// assert_eq!(random.scale(), 2);
    /// ```
    #[inline]
    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, rand_0_10::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let (low, high) = sync_scales(*low.borrow(), *high.borrow());
        let high = Decimal::from_i128_with_scale(high.mantissa() - 1, high.scale());
        UniformSampler::new_inclusive(low, high)
    }

    /// Creates a new sampler that will yield random decimal objects between `low` and `high`.
    ///
    /// The sampler will always provide decimals at the same scale as the inputs; if the inputs
    /// have different scales, the higher scale is used.
    ///
    /// # Example
    ///
    /// ```
    /// # use rand_0_10 as rand;
    /// # use rand::RngExt;
    /// # use rust_decimal_macros::dec;
    /// let mut rng = rand::rng();
    /// let random = rng.random_range(dec!(1.00)..=dec!(2.00));
    /// assert!(random >= dec!(1.00));
    /// assert!(random <= dec!(2.00));
    /// assert_eq!(random.scale(), 2);
    /// ```
    #[inline]
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, rand_0_10::distr::uniform::Error>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let (low, high) = sync_scales(*low.borrow(), *high.borrow());

        Ok(Self {
            mantissa_sampler: UniformInt::new_inclusive(low.mantissa(), high.mantissa())?,
            scale: low.scale(),
        })
    }

    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        let mantissa = self.mantissa_sampler.sample(rng);
        Decimal::from_i128_with_scale(mantissa, self.scale)
    }
}

#[inline]
fn sync_scales(mut a: Decimal, mut b: Decimal) -> (Decimal, Decimal) {
    if a.scale() == b.scale() {
        return (a, b);
    }

    a.rescale(a.scale().max(b.scale()));
    b.rescale(a.scale().max(b.scale()));

    if a.scale() != b.scale() {
        a.rescale(a.scale().min(b.scale()));
        b.rescale(a.scale().min(b.scale()));
    }

    (a, b)
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
        let random: [Decimal; 32] = rng.random();
        assert!(random.windows(2).any(|slice| { slice[0] != slice[1] }));
    }

    #[test]
    fn generates_within_range() {
        let mut rng = rng();
        for _ in 0..128 {
            let random = rng.random_range(dec!(1.00)..dec!(1.05));
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
            let random = rng.random_range(dec!(1.00)..=dec!(1.01));
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
}
