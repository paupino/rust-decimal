use crate::Decimal;
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
#[cfg(feature = "serde-with-arbitrary-precision")]
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
#[cfg(feature = "serde-with-arbitrary-precision")]
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
#[cfg(feature = "serde-with-float")]
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
#[cfg(feature = "serde-with-float")]
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
#[cfg(feature = "serde-with-str")]
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
#[cfg(feature = "serde-with-str")]
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

#[cfg(not(any(feature = "serde-str", feature = "serde-lexical")))]
impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(DecimalVisitor)
    }
}

#[cfg(all(feature = "serde-str", not(feature = "serde-float")))]
impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

#[cfg(all(feature = "serde-str", feature = "serde-float"))]
impl<'de> serde::Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_f64(DecimalVisitor)
    }
}

// It's a shame this needs to be redefined for this feature and not able to be referenced directly
#[cfg(feature = "serde-with-arbitrary-precision")]
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

    fn visit_f64<E>(self, value: f64) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        Decimal::from_str(&value.to_string()).map_err(|_| E::invalid_value(Unexpected::Float(value), &self))
    }

    fn visit_str<E>(self, value: &str) -> Result<Decimal, E>
    where
        E: serde::de::Error,
    {
        Decimal::from_str(value)
            .or_else(|_| Decimal::from_scientific(value))
            .map_err(|_| E::invalid_value(Unexpected::Str(value), &self))
    }

    #[cfg(feature = "serde-with-arbitrary-precision")]
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

#[cfg(any(feature = "serde-with-float", feature = "serde-with-arbitrary-precision"))]
struct OptionDecimalVisitor;

#[cfg(any(feature = "serde-with-float", feature = "serde-with-arbitrary-precision"))]
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

    #[cfg(all(feature = "serde-str", feature = "serde-float"))]
    fn visit_some<D>(self, d: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        // We've got multiple types that we may see so we need to use any
        d.deserialize_any(DecimalVisitor).map(Some)
    }

    #[cfg(not(all(feature = "serde-str", feature = "serde-float")))]
    fn visit_some<D>(self, d: D) -> Result<Option<Decimal>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        <Decimal as serde::Deserialize>::deserialize(d).map(Some)
    }
}

#[cfg(feature = "serde-with-str")]
struct OptionDecimalStrVisitor;

#[cfg(feature = "serde-with-str")]
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
                    .or_else(|_| Decimal::from_scientific(v))
                    .map_err(serde::de::Error::custom)?;
                Ok(Some(d))
            }
        }
    }
}

#[cfg(feature = "serde-with-arbitrary-precision")]
struct DecimalKey;

#[cfg(feature = "serde-with-arbitrary-precision")]
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

#[cfg(feature = "serde-with-arbitrary-precision")]
pub struct DecimalFromString {
    pub value: Decimal,
}

#[cfg(feature = "serde-with-arbitrary-precision")]
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
                    .or_else(|_| Decimal::from_scientific(value))
                    .map_err(serde::de::Error::custom)?;
                Ok(DecimalFromString { value: d })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(not(any(feature = "serde-float", feature = "serde-lexical")))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = crate::str::to_str_internal(self, true, None);
        serializer.serialize_str(value.0.as_ref())
    }
}

#[cfg(all(feature = "serde-float", not(feature = "serde-arbitrary-precision")))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use num_traits::ToPrimitive;
        serializer.serialize_f64(self.to_f64().unwrap())
    }
}

#[cfg(all(feature = "serde-float", feature = "serde-arbitrary-precision"))]
impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::Number::from_str(&self.to_string())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

/// Serialize/deserialize Decimals preserving the lexical order in the serialized form.
/// This is particularly useful to keep the decimal ordered when stored as a key in a key value store.
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use rust_decimal::Decimal;
/// # use std::str::FromStr;
///
/// #[derive(Serialize, Deserialize)]
/// pub struct LexicalExample {
///     #[serde(with = "rust_decimal::serde::lexical")]
///     value: Decimal,
/// }
///
/// let value1 = LexicalExample { value: Decimal::from_str("123.45").unwrap() };
/// let value2 = LexicalExample { value: Decimal::from_str("678.123").unwrap() };
/// let bin1 = bincode::serialize(&value1).unwrap();
/// let bin2 = bincode::serialize(&value2).unwrap();
/// assert!(bin1 < bin2);
///
/// ```
#[cfg(feature = "serde-lexical")]
pub mod lexical {
    use std::io::{Cursor, Read, Write};
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::Visitor;
    use super::*;

    impl Serialize for Decimal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut buffer = Vec::new();
            self.write_to(&mut buffer).map_err(serde::ser::Error::custom)?;
            serializer.serialize_bytes(&buffer)
        }
    }

    impl<'de> Deserialize<'de> for Decimal {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct DecimalVisitor;

            impl<'de> Visitor<'de> for DecimalVisitor {
                type Value = Decimal;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a byte array representing a Decimal")
                }

                fn visit_bytes<E>(self, value: &[u8]) -> Result<Decimal, E>
                where
                    E: de::Error,
                {
                    let mut cursor = Cursor::new(value);
                    Decimal::read_from(&mut cursor).map_err(de::Error::custom)
                }
            }

            deserializer.deserialize_bytes(DecimalVisitor)
        }
    }

    impl Decimal {
        // Method to write Decimal to a writer (e.g., Vec<u8>)
        fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
            let is_negative = self.is_sign_negative();
            let sign_byte = if is_negative { 0x00 } else { 0xFF };
            // Separate the decimal into integer and fractional parts
            let value = self.normalize().abs();
            let integer_part = value.trunc();
            let fractional_part = value - integer_part;
            // The fractional part is scaled up to use the largest possible precision.
            let fractional_part= if !fractional_part.is_zero() {
                fractional_part.mantissa() * 10_i128.pow(28 - fractional_part.scale())
            } else {
                0i128
            };

            // Prepare 96-bit integer (maximum Decimal precision) and fractional parts
            let mut integer_bytes = Self::serialize_i96(integer_part.mantissa());
            let mut fractional_bytes = Self::serialize_i96(fractional_part);
            if is_negative {
                for byte in &mut integer_bytes {
                    *byte = !*byte;
                }
                for byte in &mut fractional_bytes {
                    *byte = !*byte;
                }
            }
            //
            writer.write_all(&[sign_byte])?;
            writer.write_all(&integer_bytes)?;
            writer.write_all(&fractional_bytes)?;
            Ok(())
        }

        // Method to read Decimal from a reader (e.g., &[u8])
        fn read_from<R: Read>(reader: &mut R) -> Result<Decimal, std::io::Error> {
            let mut sign_byte = [0u8; 1];
            let mut integer_bytes = [0u8; 12];
            let mut fractional_bytes = [0u8; 12];

            // Read the sign byte, integer part, and fractional part from the reader
            reader.read_exact(&mut sign_byte)?;
            reader.read_exact(&mut integer_bytes)?;
            reader.read_exact(&mut fractional_bytes)?;

            let is_negative =sign_byte[0] == 0x00;

            if is_negative {
                for byte in &mut integer_bytes {
                    *byte = !*byte;
                }
                for byte in &mut fractional_bytes {
                    *byte = !*byte;
                }
            }

            // Convert the 96-bit big-endian byte arrays back to u128
            let integer_part = Self::deserialize_i96( integer_bytes);
            let fractional_part = Self::deserialize_i96( fractional_bytes);

            // Convert integer and fractional parts back to Decimal
            let i = Decimal::from_i128_with_scale(integer_part, 0);
            let f =Decimal::from_i128_with_scale(fractional_part, 28);

            // Reconstitute the decimal value
            let mut value = i + f;
            // Apply the sign
            if is_negative {
                value.set_sign_negative(true);
            }
            Ok(value)
        }

        /// Serializes an i128 to a 12 bytes array, preserving the lexicographical order.
        fn serialize_i96(value: i128) -> [u8; 12] {
            let mut buf = [0u8; 12];
            // Take the last 12 bytes (96 bits)
            buf.copy_from_slice(&value.to_be_bytes()[4..]);
            buf
        }

        /// Deserializes an i128 from a 12 bytes array, restoring the original value.
        fn deserialize_i96( buf: [u8; 12]) -> i128 {
            let mut int_buf =  [0x00; 16];
            int_buf[4..].copy_from_slice(&buf);
            i128::from_be_bytes(int_buf)
        }

    }

}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct Record {
        amount: Decimal,
    }

    #[test]
    #[cfg(not(any(feature = "serde-str", feature = "serde-lexical")))]
    fn deserialize_valid_decimal() {
        let data = [
            ("{\"amount\":\"1.234\"}", "1.234"),
            ("{\"amount\":1234}", "1234"),
            ("{\"amount\":1234.56}", "1234.56"),
            ("{\"amount\":\"1.23456e3\"}", "1234.56"),
        ];
        for &(serialized, value) in data.iter() {
            let result = serde_json::from_str(serialized);
            assert!(
                result.is_ok(),
                "expected successful deserialization for {}. Error: {:?}",
                serialized,
                result.err().unwrap()
            );
            let record: Record = result.unwrap();
            assert_eq!(
                value,
                record.amount.to_string(),
                "expected: {}, actual: {}",
                value,
                record.amount
            );
        }
    }

    #[test]
    #[cfg(feature = "serde-arbitrary-precision")]
    fn deserialize_basic_decimal() {
        let d: Decimal = serde_json::from_str("1.1234127836128763").unwrap();
        // Typically, this would not work without this feature enabled due to rounding
        assert_eq!(d.to_string(), "1.1234127836128763");
    }

    #[test]
    #[should_panic]
    fn deserialize_invalid_decimal() {
        let serialized = "{\"amount\":\"foo\"}";
        let _: Record = serde_json::from_str(serialized).unwrap();
    }

    #[test]
    #[cfg(not(any(feature = "serde-float", feature = "serde-lexical")))]
    fn serialize_decimal() {
        let record = Record {
            amount: Decimal::new(1234, 3),
        };
        let serialized = serde_json::to_string(&record).unwrap();
        assert_eq!("{\"amount\":\"1.234\"}", serialized);
    }

    #[test]
    #[cfg(not(any(feature = "serde-float", feature = "serde-lexical")))]
    fn serialize_negative_zero() {
        let record = Record { amount: -Decimal::ZERO };
        let serialized = serde_json::to_string(&record).unwrap();
        assert_eq!("{\"amount\":\"-0\"}", serialized);
    }

    #[test]
    #[cfg(feature = "serde-float")]
    fn serialize_decimal() {
        let record = Record {
            amount: Decimal::new(1234, 3),
        };
        let serialized = serde_json::to_string(&record).unwrap();
        assert_eq!("{\"amount\":1.234}", serialized);
    }

    #[test]
    #[cfg(all(feature = "serde-float", feature = "serde-arbitrary-precision"))]
    fn serialize_decimal_roundtrip() {
        let record = Record {
            // 4.81 is intentionally chosen as it is unrepresentable as a floating point number, meaning this test
            // would fail if the `serde-arbitrary-precision` was not activated.
            amount: Decimal::new(481, 2),
        };
        let serialized = serde_json::to_string(&record).unwrap();
        assert_eq!("{\"amount\":4.81}", serialized);
        let deserialized: Record = serde_json::from_str(&serialized).unwrap();
        assert_eq!(record.amount, deserialized.amount);
    }

    #[test]
    #[cfg(all(feature = "serde-str", not(feature = "serde-float")))]
    fn serialize_decimal_roundtrip() {
        let record = Record {
            amount: Decimal::new(481, 2),
        };
        let serialized = serde_json::to_string(&record).unwrap();
        assert_eq!("{\"amount\":\"4.81\"}", serialized);
        let deserialized: Record = serde_json::from_str(&serialized).unwrap();
        assert_eq!(record.amount, deserialized.amount);
    }

    #[test]
    #[cfg(all(feature = "serde-str", not(feature = "serde-float")))]
    fn bincode_serialization_not_float() {
        use bincode::{deserialize, serialize};

        let data = [
            "0",
            "0.00",
            "3.14159",
            "-3.14159",
            "1234567890123.4567890",
            "-1234567890123.4567890",
            "5233.9008808150288439427720175",
            "-5233.9008808150288439427720175",
        ];
        for &raw in data.iter() {
            let value = Decimal::from_str(raw).unwrap();
            let encoded = serialize(&value).unwrap();
            let decoded: Decimal = deserialize(&encoded[..]).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(8usize + raw.len(), encoded.len());
        }
    }

    #[test]
    #[cfg(all(feature = "serde-str", feature = "serde-float"))]
    fn bincode_serialization_serde_float() {
        use bincode::{deserialize, serialize};

        let data = [
            ("0", "0"),
            ("0.00", "0.00"),
            ("3.14159", "3.14159"),
            ("-3.14159", "-3.14159"),
            ("1234567890123.4567890", "1234567890123.4568"),
            ("-1234567890123.4567890", "-1234567890123.4568"),
        ];
        for &(value, expected) in data.iter() {
            let value = Decimal::from_str(value).unwrap();
            let expected = Decimal::from_str(expected).unwrap();
            let encoded = serialize(&value).unwrap();
            let decoded: Decimal = deserialize(&encoded[..]).unwrap();
            assert_eq!(expected, decoded);
            assert_eq!(8usize, encoded.len());
        }
    }

    #[test]
    #[cfg(all(feature = "serde-str", not(feature = "serde-float")))]
    fn bincode_nested_serialization() {
        // Issue #361
        #[derive(Deserialize, Serialize, Debug)]
        pub struct Foo {
            value: Decimal,
        }

        let s = Foo {
            value: Decimal::new(-1, 3).round_dp(0),
        };
        let ser = bincode::serialize(&s).unwrap();
        let des: Foo = bincode::deserialize(&ser).unwrap();
        assert_eq!(des.value, s.value);
    }

    #[test]
    #[cfg(feature = "serde-with-arbitrary-precision")]
    fn with_arbitrary_precision() {
        #[derive(Serialize, Deserialize)]
        pub struct ArbitraryExample {
            #[serde(with = "crate::serde::arbitrary_precision")]
            value: Decimal,
        }

        let value = ArbitraryExample {
            value: Decimal::from_str("123.400").unwrap(),
        };
        assert_eq!(&serde_json::to_string(&value).unwrap(), r#"{"value":123.400}"#);
    }

    #[test]
    #[cfg(feature = "serde-with-arbitrary-precision")]
    fn with_arbitrary_precision_from_string() {
        #[derive(Serialize, Deserialize)]
        pub struct ArbitraryExample {
            #[serde(with = "crate::serde::arbitrary_precision")]
            value: Decimal,
        }

        let value: ArbitraryExample = serde_json::from_str(r#"{"value":"1.1234127836128763"}"#).unwrap();
        assert_eq!(value.value.to_string(), "1.1234127836128763");
    }

    #[test]
    #[cfg(feature = "serde-with-float")]
    fn with_float() {
        #[derive(Serialize, Deserialize)]
        pub struct FloatExample {
            #[serde(with = "crate::serde::float")]
            value: Decimal,
        }

        let value = FloatExample {
            value: Decimal::from_str("123.400").unwrap(),
        };
        assert_eq!(&serde_json::to_string(&value).unwrap(), r#"{"value":123.4}"#);
    }

    #[test]
    #[cfg(feature = "serde-with-str")]
    fn with_str() {
        #[derive(Serialize, Deserialize)]
        pub struct StringExample {
            #[serde(with = "crate::serde::str")]
            value: Decimal,
        }

        let value = StringExample {
            value: Decimal::from_str("123.400").unwrap(),
        };
        assert_eq!(&serde_json::to_string(&value).unwrap(), r#"{"value":"123.400"}"#);
    }

    #[test]
    #[cfg(feature = "serde-with-str")]
    fn with_str_bincode() {
        use bincode::{deserialize, serialize};

        #[derive(Serialize, Deserialize)]
        struct BincodeExample {
            #[serde(with = "crate::serde::str")]
            value: Decimal,
        }

        let data = [
            ("0", "0"),
            ("0.00", "0.00"),
            ("1.234", "1.234"),
            ("3.14159", "3.14159"),
            ("-3.14159", "-3.14159"),
            ("1234567890123.4567890", "1234567890123.4567890"),
            ("-1234567890123.4567890", "-1234567890123.4567890"),
        ];
        for &(value, expected) in data.iter() {
            let value = Decimal::from_str(value).unwrap();
            let expected = Decimal::from_str(expected).unwrap();
            let input = BincodeExample { value };

            let encoded = serialize(&input).unwrap();
            let decoded: BincodeExample = deserialize(&encoded[..]).unwrap();
            assert_eq!(expected, decoded.value);
        }
    }

    #[test]
    #[cfg(feature = "serde-with-str")]
    fn with_str_bincode_optional() {
        use bincode::{deserialize, serialize};

        #[derive(Serialize, Deserialize)]
        struct BincodeExample {
            #[serde(with = "crate::serde::str_option")]
            value: Option<Decimal>,
        }

        // Some(value)
        let value = Some(Decimal::new(1234, 3));
        let input = BincodeExample { value };
        let encoded = serialize(&input).unwrap();
        let decoded: BincodeExample = deserialize(&encoded[..]).unwrap();
        assert_eq!(value, decoded.value, "Some(value)");

        // None
        let input = BincodeExample { value: None };
        let encoded = serialize(&input).unwrap();
        let decoded: BincodeExample = deserialize(&encoded[..]).unwrap();
        assert_eq!(None, decoded.value, "None");
    }

    #[test]
    #[cfg(feature = "serde-with-str")]
    fn with_str_optional() {
        #[derive(Serialize, Deserialize)]
        pub struct StringExample {
            #[serde(with = "crate::serde::str_option")]
            value: Option<Decimal>,
        }

        let original = StringExample {
            value: Some(Decimal::from_str("123.400").unwrap()),
        };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":"123.400"}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":"123.400"}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_some());
        assert_eq!(deserialized.value.unwrap().unpack(), original.value.unwrap().unpack());

        // Null tests
        let original = StringExample { value: None };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":null}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_none());

        // Empty string deserialization tests
        let original = StringExample { value: None };
        let deserialized: StringExample = serde_json::from_str(r#"{"value":""}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_none());
    }

    #[test]
    #[cfg(feature = "serde-with-float")]
    fn with_float_optional() {
        #[derive(Serialize, Deserialize)]
        pub struct StringExample {
            #[serde(with = "crate::serde::float_option")]
            value: Option<Decimal>,
        }

        let original = StringExample {
            value: Some(Decimal::from_str("123.400").unwrap()),
        };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":123.4}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":123.4}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_some()); // Scale is different!

        // Null tests
        let original = StringExample { value: None };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":null}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_none());
    }

    #[test]
    #[cfg(feature = "serde-with-arbitrary-precision")]
    fn with_arbitrary_precision_optional() {
        #[derive(Serialize, Deserialize)]
        pub struct StringExample {
            #[serde(with = "crate::serde::arbitrary_precision_option")]
            value: Option<Decimal>,
        }

        let original = StringExample {
            value: Some(Decimal::from_str("123.400").unwrap()),
        };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":123.400}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":123.400}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_some());
        assert_eq!(deserialized.value.unwrap().unpack(), original.value.unwrap().unpack());

        // Null tests
        let original = StringExample { value: None };
        assert_eq!(&serde_json::to_string(&original).unwrap(), r#"{"value":null}"#);
        let deserialized: StringExample = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(deserialized.value, original.value);
        assert!(deserialized.value.is_none());
    }

    #[test]
    #[cfg(feature = "serde-lexical")]
    fn serialize_decimal_lexical() {
        let data = [
            Decimal::from_i128_with_scale(-1000, 0),
            Decimal::from_i128_with_scale(-100, 0),
            Decimal::from_i128_with_scale(-10, 0),
            Decimal::from_i128_with_scale(-31415926535897932384626433833, 28),
            Decimal::from_i128_with_scale(-2, 0),
            Decimal::from_i128_with_scale(-102, 2),
            Decimal::from_i128_with_scale(-101, 2),
            Decimal::from_i128_with_scale(-10000000000000000000000000002, 28),
            Decimal::from_i128_with_scale(-10000000000000000000000000001, 28),
            Decimal::NEGATIVE_ONE,
            Decimal::from_i128_with_scale(-2, 28),
            Decimal::from_i128_with_scale(-1, 28),
            -Decimal::ZERO,
            Decimal::ZERO,
            Decimal::from_i128_with_scale(1, 28),
            Decimal::from_i128_with_scale(2, 28),
            Decimal::ONE,
            Decimal::from_i128_with_scale(10000000000000000000000000001, 28),
            Decimal::from_i128_with_scale(10000000000000000000000000002, 28),
            Decimal::from_i128_with_scale(101, 2),
            Decimal::from_i128_with_scale(102, 2),
            Decimal::TWO,
            Decimal::from_i128_with_scale(31415926535897932384626433833, 28),
            Decimal::TEN,
            Decimal::ONE_HUNDRED,
            Decimal::ONE_THOUSAND,
            Decimal::MAX,
        ];
        let mut previous = bincode::serialize(&Decimal::MIN).unwrap();
        for value in data {
            let encoded = bincode::serialize(&value).unwrap();
            let decoded: Decimal = bincode::deserialize(&encoded[..]).unwrap();
            assert_eq!(value, decoded);
            assert!(previous <= encoded, "{value} -> {previous:?} <= {encoded:?}");
            previous = encoded;
        }
    }
}
