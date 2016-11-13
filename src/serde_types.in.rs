impl serde::Deserialize for Decimal {
    fn deserialize<D>(deserializer: &mut D) -> Result<Decimal, D::Error>
        where D: serde::de::Deserializer {
        deserializer.deserialize(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl serde::de::Visitor for DecimalVisitor {
    type Value = Decimal;

    fn visit_i16<E>(&mut self, value: i16) -> Result<Decimal, E>
        where E: serde::Error {
        match Decimal::from_i32(value as i32) {
            Some(s) => Ok(s),
            None => Ok(Decimal::zero()),
        }
    }

    fn visit_i32<E>(&mut self, value: i32) -> Result<Decimal, E>
        where E: serde::Error {
        match Decimal::from_i32(value) {
            Some(s) => Ok(s),
            None => Ok(Decimal::zero()),
        }
    }

    fn visit_str<E>(&mut self, value: &str) -> Result<Decimal, E>
        where E: serde::Error {
        match Decimal::from_str(value) {
            Ok(s) => Ok(s),
            Err(_) => Ok(Decimal::zero()),
        }
    }
}

impl serde::Serialize for Decimal {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer {
        serializer.serialize_str(&(self.to_string())[..])
    }
}
