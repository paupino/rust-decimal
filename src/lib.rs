extern crate num;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "postgres")]
#[macro_use]
extern crate postgres as pg_crate;

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde")]
#[cfg(test)]
extern crate serde_json;
#[cfg(feature = "serde")]
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod decimal;
mod error;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "serde")]
mod serde_types;

pub use decimal::Decimal;
pub use error::Error;
