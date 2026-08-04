#![cfg(feature = "serde")]

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    amount: Decimal,
}

#[test]
#[cfg(not(feature = "serde-str"))]
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
    let s = "1.1234127836128763";
    let d: Decimal = serde_json::from_str(s).unwrap();
    // Typically, this would not work without this feature enabled due to rounding
    assert_eq!(d.to_string(), s);
}

#[test]
#[cfg(feature = "serde-arbitrary-precision")]
fn deserialize_f64_scientific_notation_from_value() {
    // When serde_json's arbitrary_precision is enabled, small floats that roundtrip
    // through f64 are deserialized via visit_f64. zmij/ryu formats them in scientific
    // notation (e.g. "5.06e-6") which Decimal::from_str doesn't support.
    // This test ensures visit_f64 falls back to from_scientific.
    let json: serde_json::Value = serde_json::from_str(r#"{"amount": 5.06e-6}"#).unwrap();
    let record: Record = serde_json::from_value(json).unwrap();
    assert_eq!(record.amount.to_string(), "0.00000506");
}

#[test]
#[should_panic]
fn deserialize_invalid_decimal() {
    let serialized = "{\"amount\":\"foo\"}";
    let _: Record = serde_json::from_str(serialized).unwrap();
}

#[test]
#[cfg(not(feature = "serde-float"))]
fn serialize_decimal() {
    let record = Record {
        amount: Decimal::new(1234, 3),
    };
    let serialized = serde_json::to_string(&record).unwrap();
    assert_eq!("{\"amount\":\"1.234\"}", serialized);
}

#[test]
#[cfg(not(feature = "serde-float"))]
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
#[cfg(all(feature = "serde-float", feature = "serde-arbitrary-precision"))]
fn serialize_whole_number_decimal() {
    let data = [
        ("0", "0"),
        ("1.0", "1.0"),
        ("0.00", "0.00"),
        ("1.234", "1.234"),
        ("3.14159", "3.14159"),
        ("-3.14159", "-3.14159"),
        ("1234567890123.4567890", "1234567890123.4567890"),
        ("-1234567890123.4567890", "-1234567890123.4567890"),
    ];

    for &(value, expected) in data.iter() {
        let record = Record {
            amount: Decimal::from_str(value).unwrap(),
        };

        let serialized = serde_json::to_string(&record).unwrap();
        let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let deserialized: Record = serde_json::from_value(value).unwrap();

        assert_eq!(expected, deserialized.amount.to_string());
    }
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
#[cfg(not(feature = "serde-float"))]
fn bincode_roundtrip_without_serde_str() {
    // bincode is not self-describing, so `deserialize_any` cannot work.
    // This must succeed without any opt-in feature.
    let expected = Decimal::from_str("12.34").unwrap();
    let encoded = bincode::serialize(&expected).unwrap();
    let decoded: Decimal = bincode::deserialize(&encoded).unwrap();
    assert_eq!(expected, decoded);
    assert_eq!(expected.scale(), decoded.scale());
}

#[test]
#[cfg(not(feature = "serde-float"))]
fn bincode_roundtrip_preserves_scale() {
    let expected = Decimal::from_str("1.0000").unwrap();
    let encoded = bincode::serialize(&expected).unwrap();
    let decoded: Decimal = bincode::deserialize(&encoded).unwrap();
    assert_eq!(4, decoded.scale());
    assert_eq!("1.0000", decoded.to_string());
}

#[test]
#[cfg(not(feature = "serde-float"))]
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
#[cfg(not(feature = "serde-float"))]
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
        #[serde(with = "rust_decimal::serde::arbitrary_precision")]
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
        #[serde(with = "rust_decimal::serde::arbitrary_precision")]
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
        #[serde(with = "rust_decimal::serde::float")]
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
        #[serde(with = "rust_decimal::serde::str")]
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
        #[serde(with = "rust_decimal::serde::str")]
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
        #[serde(with = "rust_decimal::serde::str_option")]
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
        #[serde(with = "rust_decimal::serde::str_option")]
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
#[cfg(feature = "serde-with-str")]
fn with_str_tagged_enum_optional() {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum TaggedEnumExample {
        Example {
            #[serde(with = "rust_decimal::serde::str_option")]
            value: Option<Decimal>,
        },
    }

    let original = TaggedEnumExample::Example { value: None };
    let expected = r#"{"kind":"example","value":null}"#;

    let serialized = serde_json::to_string(&original).unwrap();
    assert_eq!(serialized, expected);

    let deserialized: TaggedEnumExample = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, original);
}

#[test]
#[cfg(feature = "serde-with-float")]
fn with_float_optional() {
    #[derive(Serialize, Deserialize)]
    pub struct StringExample {
        #[serde(with = "rust_decimal::serde::float_option")]
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
#[cfg(feature = "serde-with-float")]
fn with_float_tagged_enum_optional() {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum TaggedEnumExample {
        Example {
            #[serde(with = "rust_decimal::serde::float_option")]
            value: Option<Decimal>,
        },
    }

    let original = TaggedEnumExample::Example { value: None };
    let expected = r#"{"kind":"example","value":null}"#;

    let serialized = serde_json::to_string(&original).unwrap();
    assert_eq!(serialized, expected);

    let deserialized: TaggedEnumExample = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, original);
}

#[test]
#[cfg(feature = "serde-with-arbitrary-precision")]
fn with_arbitrary_precision_optional() {
    #[derive(Serialize, Deserialize)]
    pub struct StringExample {
        #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
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
#[cfg(feature = "serde-with-arbitrary-precision")]
fn with_arbitrary_precision_tagged_enum_optional() {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    pub enum TaggedEnumExample {
        Example {
            #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
            value: Option<Decimal>,
        },
    }

    let original = TaggedEnumExample::Example { value: None };
    let expected = r#"{"kind":"example","value":null}"#;

    let serialized = serde_json::to_string(&original).unwrap();
    assert_eq!(serialized, expected);

    let deserialized: TaggedEnumExample = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, original);
}
