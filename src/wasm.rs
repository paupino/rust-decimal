use alloc::string::String;
use core::str::FromStr;

use num_traits::{FromPrimitive, ToPrimitive};
use wasm_bindgen::prelude::*;

use crate::Decimal;

#[wasm_bindgen]
impl Decimal {
    /// Returns a new `Decimal` object instance by converting a primitive number.
    ///
    /// Returns `undefined` in JavaScript if the value cannot be represented as a `Decimal`
    /// (e.g. `NaN`, `Infinity`, or `-Infinity`).
    #[wasm_bindgen(js_name = fromNumber)]
    #[must_use]
    pub fn from_number(value: f64) -> Option<Decimal> {
        Decimal::from_f64(value)
    }

    /// Returns a new `Decimal` object instance by parsing a string representation.
    ///
    /// Returns `undefined` in JavaScript if the string cannot be parsed as a valid `Decimal`.
    #[wasm_bindgen(js_name = fromString)]
    #[must_use]
    pub fn from_string(value: &str) -> Option<Decimal> {
        Decimal::from_str(value).ok()
    }

    /// Returns the value of this `Decimal` converted to a primitive number.
    ///
    /// # Note
    /// At the time of writing, the conversion from `Decimal` to `f64` cannot fail. To guard
    /// against future implementation changes, `f64::NAN` is returned as a fallback value.
    #[wasm_bindgen(js_name = toNumber)]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        self.to_f64().unwrap_or(f64::NAN)
    }

    /// Returns the string representation of this `Decimal`.
    ///
    /// This intentionally overrides the default JS `toString()` so that string coercion
    /// (e.g. template literals, `console.log`) produces the decimal representation.
    #[wasm_bindgen(js_name = toString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.to_string()
    }
}
