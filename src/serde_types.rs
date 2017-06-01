use Decimal;

use num::{FromPrimitive, Zero};
use serde;

use std::fmt;
use std::str::FromStr;

impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
        where D: serde::de::Deserializer<'de> {
        deserializer.deserialize_any(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn visit_i16<E>(self, value: i16) -> Result<Decimal, E> {
        match Decimal::from_i32(value as i32) {
            Some(s) => Ok(s),
            None => Ok(Decimal::zero()),
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Decimal, E> {
        match Decimal::from_i32(value) {
            Some(s) => Ok(s),
            None => Ok(Decimal::zero()),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Decimal, E> {
        match Decimal::from_str(value) {
            Ok(s) => Ok(s),
            Err(_) => Ok(Decimal::zero()),
        }
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a Decimal type representing a fixed-point number")
    }
}

impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
