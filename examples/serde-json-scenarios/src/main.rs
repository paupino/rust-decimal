use rust_decimal::prelude::*;

type ExampleResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> ExampleResult {
    demonstrate_default_behavior()?;
    demonstrate_arbitrary_precision_deserialization_with_string_serialization()?;
    Ok(())
}

/// The default behavior of the library always expects string results. That is, it will serialize the
/// Decimal as string, but also expect a string when deserializing.
/// Note: this is not enough for bincode representations since there is no deserialization hint that the
/// field is a string.
fn demonstrate_default_behavior() -> ExampleResult {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Total {
        value: Decimal,
    }
    let total = Total { value: dec!(1.23) };
    let serialized = serde_json::to_string(&total)?;
    assert_eq!(r#"{"value":"1.23"}"#, serialized);

    // If we try to deserialize the same string we should succeed
    let deserialized: Total = serde_json::from_str(&serialized)?;
    assert_eq!(dec!(1.23), deserialized.value);

    // Technically, by default we also support deserializing from a number, however this is doing a float
    // conversion and is not recommended.
    let deserialized: Total = serde_json::from_str(r#"{"value":1.23}"#)?;
    assert_eq!(dec!(1.23), deserialized.value);
    Ok(())
}

/// This demonstrates using arbitrary precision for a decimal value - even though the
/// default string serialization behavior is baked in.
fn demonstrate_arbitrary_precision_deserialization_with_string_serialization() -> ExampleResult {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Total {
        #[serde(deserialize_with = "rust_decimal::serde::arbitrary_precision::deserialize")]
        value: Decimal,
    }

    let total = Total { value: dec!(1.23) };
    let serialized = serde_json::to_string(&total)?;
    assert_eq!(r#"{"value":"1.23"}"#, serialized);

    // If we try to deserialize the same string we should succeed
    let deserialized: Total = serde_json::from_str(&serialized)?;
    assert_eq!(dec!(1.23), deserialized.value);

    // If we try to deserialize a float then this will succeed as well
    let deserialized: Total = serde_json::from_str(r#"{"value":1.23}"#)?;
    assert_eq!(dec!(1.23), deserialized.value);
    Ok(())
}
