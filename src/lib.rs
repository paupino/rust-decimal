extern crate num;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "postgres")]
#[macro_use]
extern crate postgres as pg_crate;

mod decimal;
mod error;
mod serde_types;

#[cfg(feature = "postgres")]
mod postgres;

pub use decimal::Decimal;
pub use error::Error;
