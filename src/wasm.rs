use num_traits::{FromPrimitive, ToPrimitive};
use wasm_bindgen::prelude::*;

use crate::Decimal;

#[wasm_bindgen]
impl Decimal {
    /// Returns a new `Decimal` object instance by converting a primitive number.
    #[wasm_bindgen(js_name = fromNumber)]
    #[must_use]
    pub fn from_number(value: f64) -> Option<Decimal> {
        Decimal::from_f64(value)
    }

    /// Returns the value of this `Decimal` converted to a primitive number.
    ///
    /// # Caution
    /// At the time of writing this implementation the conversion from `Decimal` to `f64` cannot
    /// fail. To prevent undefined behavior in case the underlying implementation changes `f64::NAN`
    /// is returned as a stable fallback value.
    #[wasm_bindgen(js_name = toNumber)]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        self.to_f64().unwrap_or(f64::NAN)
    }
}
