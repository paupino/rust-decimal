use crate::Decimal;

use proptest::arbitrary::{Arbitrary, StrategyFor};
use proptest::prelude::*;
use proptest::strategy::FilterMap;

impl Arbitrary for Decimal {
    type Parameters = ();
    type Strategy = FilterMap<StrategyFor<(u32, u32, u32, bool, u8)>, fn((u32, u32, u32, bool, u8)) -> Option<Self>>;

    fn arbitrary_with(_parameters: Self::Parameters) -> Self::Strategy {
        // generate 3 arbitrary u32, a bool and an u32 between 0 to 28
        any::<(u32, u32, u32, bool, u8)>().prop_filter_map(
            "scale must be between 0..28",
            |(lo, mid, hi, negative, scale)| {
                if scale <= 28 {
                    Some(Decimal::from_parts(lo, mid, hi, negative, scale as u32))
                } else {
                    None
                }
            },
        )
    }
}
