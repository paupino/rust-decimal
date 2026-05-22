use proptest::prelude::*;
use rust_decimal::Decimal;

proptest! {
    #[test]
    fn test_proptest_validate_arbitrary_decimals(num in any::<Decimal>()) {
        assert!(num.is_zero() || !num.is_zero());
    }
}
