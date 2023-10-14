use num_traits::{FromPrimitive, ToPrimitive};
use wasm_bindgen::prelude::wasm_bindgen;

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
    #[wasm_bindgen(js_name = toNumber)]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        self.to_f64().unwrap_or(f64::NAN)
    }
}
