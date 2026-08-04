use crate::Decimal;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::ToString;
use core::{fmt, str::FromStr};
use num_traits::FromPrimitive;
use serde::{self, de::Unexpected};

/// Serialize/deserialize Decimals as arbitrary precision numbers in JSON using the `arbitrary_precision` feature within `serde_json`.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct ArbitraryExample {
///     #[serde(with = "rust_decimal::serde::arbitrary_precision")]
///     value: Decimal,
/// }
///
/// let value = ArbitraryExample { value: Decimal::from_str("123.400").unwrap() };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":123.400}"#
/// );
/// ```
#[cfg(feature = "serde-default-exact")]
pub mod arbitrary_precision {
    use super::*;
    use serde::Serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(DecimalVisitor)
    }

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::Number::from_str(&value.to_string())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

/// Serialize/deserialize optional Decimals as arbitrary precision numbers in JSON using the `arbitrary_precision` feature within `serde_json`.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct ArbitraryExample {
///     #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
///     value: Option<Decimal>,
/// }
///
/// let value = ArbitraryExample { value: Some(Decimal::from_str("123.400").unwrap()) };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":123.400}"#
/// );
///
/// let value = ArbitraryExample { value: None };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":null}"#
/// );
/// ```
#[cfg(feature = "serde-default-exact")]
pub mod arbitrary_precision_option {
    use super::*;
    use serde::Serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionDecimalVisitor)
    }

    pub fn serialize<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *value {
            Some(ref decimal) => serde_json::Number::from_str(&decimal.to_string())
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

/// Serialize/deserialize Decimals as floats.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct FloatExample {
///     #[serde(with = "rust_decimal::serde::float")]
///     value: Decimal,
/// }
///
/// let value = FloatExample { value: Decimal::from_str("123.400").unwrap() };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":123.4}"#
/// );
/// ```
pub mod float {
    use super::*;
    use serde::Serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(DecimalVisitor)
    }

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use num_traits::ToPrimitive;
        value.to_f64().unwrap().serialize(serializer)
    }
}

/// Serialize/deserialize optional Decimals as floats.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct FloatExample {
///     #[serde(with = "rust_decimal::serde::float_option")]
///     value: Option<Decimal>,
/// }
///
/// let value = FloatExample { value: Some(Decimal::from_str("123.400").unwrap()) };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":123.4}"#
/// );
///
/// let value = FloatExample { value: None };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":null}"#
/// );
/// ```
pub mod float_option {
    use super::*;
    use serde::Serialize;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionDecimalVisitor)
    }

    pub fn serialize<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *value {
            Some(ref decimal) => {
                use num_traits::ToPrimitive;
                decimal.to_f64().unwrap().serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }
}

/// Serialize/deserialize Decimals as strings. This is particularly useful when using binary encoding formats.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct StringExample {
///     #[serde(with = "rust_decimal::serde::str")]
///     value: Decimal,
/// }
///
/// let value = StringExample { value: Decimal::from_str("123.400").unwrap() };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":"123.400"}"#
/// );
///
/// ```
pub mod str {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = crate::str::to_str_internal(value, true, None);
        serializer.serialize_str(value.0.as_ref())
    }
}

/// Serialize/deserialize optional Decimals as strings. This is particularly useful when using binary encoding formats.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct StringExample {
///     #[serde(with = "rust_decimal::serde::str_option")]
///     value: Option<Decimal>,
/// }
///
/// let value = StringExample { value: Some(Decimal::from_str("123.400").unwrap()) };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":"123.400"}"#
/// );
///
/// let value = StringExample { value: None };
/// assert_eq!(
///     &serde_json::to_string(&value).unwrap(),
///     r#"{"value":null}"#
/// );
/// ```
pub mod str_option {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionDecimalStrVisitor)
    }

    pub fn serialize<S>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *value {
            Some(ref decimal) => {
                let decimal = crate::str::to_str_internal(decimal, true, None);
                serializer.serialize_some::<str>(decimal.0.as_ref())
            }
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // Self-describing formats (JSON, YAML) can accept either a string or a
        // number, so `deserialize_any` is both safe and maximally permissive.
        // Non-self-describing formats (bincode, postcard) reject `deserialize_any`
        // outright and must be told the concrete type up front.
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(DecimalVisitor)
        } else {
            #[cfg(feature = "serde-default-number")]
            {
                deserializer.deserialize_f64(DecimalVisitor)
            }
            #[cfg(not(feature = "serde-default-number"))]
            {
                deserializer.deserialize_str(DecimalVisitor)
            }
        }
    }
}

// It's a shame this needs to be redefined for this feature and not able to be referenced directly
#[cfg(feature = "serde-default-exact")]
const DECIMAL_KEY_TOKEN: &str = "$serde_json::private::Number";

struct DecimalVisitor;

impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a Decimal type representing a fixed-point number")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        match Decimal::from_i64(value) {
            Some(s) => Ok(s),
            None => Err(E::invalid_value(Unexpected::Signed(value), &self)),
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        match Decimal::from_u64(value) {
            Some(s) => Ok(s),
            None => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }

    #[cfg(not(feature = "serde-default-exact"))]
    fn visit_f64<E>(self, value: f64) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        Decimal::from_str(&value.to_string()).map_err(|_| E::invalid_value(Unexpected::Float(value), &self))
    }

    #[cfg(feature = "serde-default-exact")]
    fn visit_f64<E>(self, value: f64) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        let mut buf = zmij::Buffer::new();
        let formatted = buf.format_finite(value);
        Decimal::from_str(formatted)
            .or_else(|_| Decimal::from_scientific_exact(formatted))
            .map_err(|_| E::invalid_value(Unexpected::Float(value), &self))
    }

    fn visit_str<E>(self, value: &str) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        Decimal::from_str(value)
            .or_else(|_| Decimal::from_scientific_exact(value))
            .map_err(|_| E::invalid_value(Unexpected::Str(value), &self))
    }

    #[cfg(feature = "serde-default-exact")]
    fn visit_map<A>(self, map: A) -> Result<Decimal, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map = map;
        let value = map.next_key::<DecimalKey>()?;
        if value.is_none() {
            return Err(serde::de::Error::invalid_type(Unexpected::Map, &self));
        }
        let v: DecimalFromString = map.next_value()?;
        Ok(v.value)
    }
}

struct OptionDecimalVisitor;

impl<'de> serde::de::Visitor<'de> for OptionDecimalVisitor {
    type Value = Option<Decimal>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Decimal type representing a fixed-point number")
    }

    fn visit_none<E>(self) -> Result<Option<Decimal>, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Option<Decimal>, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, d: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // Delegating picks up the format-aware dispatch on the main impl, which
        // handles both self-describing and binary formats correctly.
        <Decimal as serde::Deserialize>::deserialize(d).map(Some)
    }
}

struct OptionDecimalStrVisitor;

impl<'de> serde::de::Visitor<'de> for OptionDecimalStrVisitor {
    type Value = Option<Decimal>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Decimal type representing a fixed-point number")
    }

    fn visit_none<E>(self) -> Result<Option<Decimal>, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Option<Decimal>, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, d: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        d.deserialize_str(Self)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.is_empty() {
            true => Ok(None),
            false => {
                let d = Decimal::from_str(v)
                    .or_else(|_| Decimal::from_scientific_exact(v))
                    .map_err(serde::de::Error::custom)?;
                Ok(Some(d))
            }
        }
    }
}

#[cfg(feature = "serde-default-exact")]
struct DecimalKey;

#[cfg(feature = "serde-default-exact")]
impl<'de> serde::de::Deserialize<'de> for DecimalKey {
    fn deserialize<D>(deserializer: D) -> Result<DecimalKey, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid decimal field")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: serde::de::Error,
            {
                if s == DECIMAL_KEY_TOKEN {
                    Ok(())
                } else {
                    Err(serde::de::Error::custom("expected field with custom name"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(DecimalKey)
    }
}

#[cfg(feature = "serde-default-exact")]
pub struct DecimalFromString {
    pub value: Decimal,
}

#[cfg(feature = "serde-default-exact")]
impl<'de> serde::de::Deserialize<'de> for DecimalFromString {
    fn deserialize<D>(deserializer: D) -> Result<DecimalFromString, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DecimalFromString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string containing a decimal")
            }

            fn visit_str<E>(self, value: &str) -> Result<DecimalFromString, E>
            where
                E: serde::de::Error,
            {
                let d = Decimal::from_str(value)
                    .or_else(|_| Decimal::from_scientific_exact(value))
                    .map_err(serde::de::Error::custom)?;
                Ok(DecimalFromString { value: d })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(not(feature = "serde-default-number"))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = crate::str::to_str_internal(self, true, None);
        serializer.serialize_str(value.0.as_ref())
    }
}

#[cfg(all(feature = "serde-default-number", not(feature = "serde-default-exact")))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use num_traits::ToPrimitive;
        serializer.serialize_f64(self.to_f64().unwrap())
    }
}

#[cfg(all(feature = "serde-default-number", feature = "serde-default-exact"))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // `serde_json::Number` is JSON-specific: a binary format receives its
        // internal representation rather than a number. Fall back to the plain
        // numeric wire form there, matching `serde-default-number` alone.
        if !serializer.is_human_readable() {
            use num_traits::ToPrimitive;
            return serializer.serialize_f64(self.to_f64().unwrap());
        }
        serde_json::Number::from_str(&self.to_string())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
