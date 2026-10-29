//! [Diesel](https://diesel.rs) support for [`rust_decimal::Decimal`].
//!
//! Enable the `postgres` and/or `mysql` features, then map `Numeric` columns to [`DieselDecimal`],
//! either directly or via `serialize_as` / `deserialize_as`:
//!
//! ```ignore
//! use rust_decimal::Decimal;
//! use rust_decimal_diesel::DieselDecimal;
//!
//! #[derive(Queryable, Insertable)]
//! #[diesel(table_name = orders)]
//! struct Order {
//!     #[diesel(serialize_as = DieselDecimal, deserialize_as = DieselDecimal)]
//!     amount: Decimal,
//! }
//! ```
#![forbid(unsafe_code)]

use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::Numeric};
use rust_decimal::Decimal;

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;

/// A [`Decimal`] that maps to the Diesel `Numeric` SQL type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = Numeric)]
pub struct DieselDecimal(pub Decimal);

impl From<Decimal> for DieselDecimal {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<DieselDecimal> for Decimal {
    fn from(value: DieselDecimal) -> Self {
        value.0
    }
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use diesel::prelude::*;

    diesel::table! {
        orders (id) {
            id -> Integer,
            amount -> Numeric,
            fee -> Nullable<Numeric>,
        }
    }

    // Compile check for the documented `serialize_as` / `deserialize_as` usage.
    #[allow(dead_code)]
    #[derive(Queryable, Insertable)]
    #[diesel(table_name = orders, check_for_backend(diesel::pg::Pg))]
    struct Order {
        id: i32,
        #[diesel(serialize_as = DieselDecimal, deserialize_as = DieselDecimal)]
        amount: Decimal,
        fee: Option<DieselDecimal>,
    }

    #[test]
    fn filters_on_wrapper() {
        let query = orders::table.filter(orders::amount.gt(DieselDecimal(Decimal::ONE)));
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&query).to_string();
        assert!(sql.contains("\"orders\".\"amount\" > $1"), "{sql}");
    }
}
