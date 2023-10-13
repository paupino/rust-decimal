use num_traits::ToPrimitive;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::Decimal;

#[wasm_bindgen]
impl Decimal {
    /// Returns the value of this `Decimal` converted to a primitive number.
    #[wasm_bindgen(js_name = toNumber)]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        self.to_f64().unwrap_or(f64::NAN)
    }
}
