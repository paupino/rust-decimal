#![allow(clippy::disallowed_names)]

use rocket::form::{Form, FromForm};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(FromForm)]
struct Example {
    foo: Decimal,
    bar: Decimal,
}

#[test]
fn it_can_parse_form() {
    let parsed: Example = Form::parse("bar=0.12345678901234567890123456789&foo=-123456.78").unwrap();
    assert_eq!(parsed.foo, Decimal::from_str("-123456.78").unwrap());
    assert_eq!(
        parsed.bar,
        Decimal::from_str("0.12345678901234567890123456789").unwrap()
    );
}
