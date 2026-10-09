use rust_decimal_macros::dec;

fn main() {
    let _ = dec!(101, radix "2");
    let _ = dec!(1, exp 1.5);
    let _ = dec!(1, exp !3);
    let _ = dec!(1, exp - 1.5);
}
