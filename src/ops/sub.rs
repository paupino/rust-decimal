pub(crate) fn sub_impl(d1: &Decimal, d2: &Decimal) -> CalculationResult {
    add_sub_internal(d1, d2, true)
}
