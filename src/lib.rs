extern crate num;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "postgres")]
#[macro_use]
extern crate postgres as pg_crate;

mod serde_types;
mod decimal;

#[cfg(feature = "postgres")]
mod postgres;

pub use decimal::Decimal;
