use rocket::form::{self, FromFormField, ValueField};
use std::str::FromStr;
use crate::Decimal;

impl<'v> FromFormField<'v> for Decimal {
    fn default() -> Option<Self> {
        Default::default()
    }
    fn from_value(field: ValueField<'v>) -> form::Result<'v, Self> {
        Decimal::from_str(field.value).map_err(|_| form::Error::validation("not a valid number").into())
    }
}
