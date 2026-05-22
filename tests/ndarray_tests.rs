use rust_decimal::Decimal;

#[test]
fn it_can_do_scalar_ops_in_ndarray() {
    use ndarray_0_16::Array1;
    use num_traits::FromPrimitive;

    let array_a = Array1::from(vec![
        Decimal::from_f32(1.0).unwrap(),
        Decimal::from_f32(2.0).unwrap(),
        Decimal::from_f32(3.0).unwrap(),
    ]);

    // Add
    let output = array_a.clone() + Decimal::from_f32(5.0).unwrap();
    let expectation = Array1::from(vec![
        Decimal::from_f32(6.0).unwrap(),
        Decimal::from_f32(7.0).unwrap(),
        Decimal::from_f32(8.0).unwrap(),
    ]);
    assert_eq!(output, expectation);

    // Sub
    let output = array_a.clone() - Decimal::from_f32(5.0).unwrap();
    let expectation = Array1::from(vec![
        Decimal::from_f32(-4.0).unwrap(),
        Decimal::from_f32(-3.0).unwrap(),
        Decimal::from_f32(-2.0).unwrap(),
    ]);
    assert_eq!(output, expectation);

    // Mul
    let output = array_a.clone() * Decimal::from_f32(5.0).unwrap();
    let expectation = Array1::from(vec![
        Decimal::from_f32(5.0).unwrap(),
        Decimal::from_f32(10.0).unwrap(),
        Decimal::from_f32(15.0).unwrap(),
    ]);
    assert_eq!(output, expectation);

    // Div
    let output = array_a / Decimal::from_f32(5.0).unwrap();
    let expectation = Array1::from(vec![
        Decimal::from_f32(0.2).unwrap(),
        Decimal::from_f32(0.4).unwrap(),
        Decimal::from_f32(0.6).unwrap(),
    ]);
    assert_eq!(output, expectation);
}
