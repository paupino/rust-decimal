use crate::Decimal;
use diesel::{
    deserialize::{self, FromSql},
    mysql::{Mysql, MysqlType, sql_types::Unsigned},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{BigInt, Double, Float, Integer, Numeric, SmallInt, TinyInt},
};
use std::io::Write;
use std::str::FromStr;

#[cfg(feature = "diesel")]
impl ToSql<Numeric, Mysql> for Decimal {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> serialize::Result {
        write!(out, "{}", *self).map(|_| IsNull::No).map_err(|e| e.into())
    }
}

#[cfg(feature = "diesel")]
impl FromSql<Numeric, Mysql> for Decimal {
    fn from_sql(numeric: diesel::mysql::MysqlValue) -> deserialize::Result<Self> {
        match numeric.value_type() {
            MysqlType::Numeric => {
                let s = std::str::from_utf8(numeric.as_bytes())?;
                Decimal::from_str(s).map_err(|e| e.into())
            }
            MysqlType::Tiny => <i8 as FromSql<TinyInt, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::UnsignedTiny => <u8 as FromSql<Unsigned<TinyInt>, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::Short => <i16 as FromSql<SmallInt, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::UnsignedShort => {
                <u16 as FromSql<Unsigned<SmallInt>, Mysql>>::from_sql(numeric).map(Decimal::from)
            }
            MysqlType::Long => <i32 as FromSql<Integer, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::UnsignedLong => <u32 as FromSql<Unsigned<Integer>, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::LongLong => <i64 as FromSql<BigInt, Mysql>>::from_sql(numeric).map(Decimal::from),
            MysqlType::UnsignedLongLong => {
                <u64 as FromSql<Unsigned<BigInt>, Mysql>>::from_sql(numeric).map(Decimal::from)
            }
            MysqlType::Float => {
                let value = <f32 as FromSql<Float, Mysql>>::from_sql(numeric)?;
                Decimal::try_from(value).map_err(Into::into)
            }
            MysqlType::Double => {
                let value = <f64 as FromSql<Double, Mysql>>::from_sql(numeric)?;
                Decimal::try_from(value).map_err(Into::into)
            }
            value_type => Err(format!("Unexpected MySQL type for numeric value: {value_type:?}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        value: Decimal,
    }

    struct NullableTest {
        value: Option<Decimal>,
    }

    pub static TEST_DECIMALS: &[(u32, u32, &str, &str)] = &[
        // precision, scale, sent, expected
        (1, 0, "1", "1"),
        (6, 2, "1", "1.00"),
        (6, 2, "9999.99", "9999.99"),
        (35, 6, "3950.123456", "3950.123456"),
        (10, 2, "3950.123456", "3950.12"),
        (35, 6, "3950", "3950.000000"),
        (4, 0, "3950", "3950"),
        (35, 6, "0.1", "0.100000"),
        (35, 6, "0.01", "0.010000"),
        (35, 6, "0.001", "0.001000"),
        (35, 6, "0.0001", "0.000100"),
        (35, 6, "0.00001", "0.000010"),
        (35, 6, "0.000001", "0.000001"),
        (35, 6, "1", "1.000000"),
        (35, 6, "-100", "-100.000000"),
        (35, 6, "-123.456", "-123.456000"),
        (35, 6, "119996.25", "119996.250000"),
        (35, 6, "1000000", "1000000.000000"),
        (35, 6, "9999999.99999", "9999999.999990"),
        (35, 6, "12340.56789", "12340.567890"),
    ];

    /// Gets the URL for connecting to MySQL for testing. Set the MYSQL_URL
    /// environment variable to change from the default of "mysql://root@localhost/mysql".
    fn get_mysql_url() -> String {
        if let Ok(url) = std::env::var("MYSQL_URL") {
            return url;
        }
        "mysql://root@127.0.0.1/mysql".to_string()
    }

    #[cfg(feature = "diesel")]
    mod diesel_tests {
        use super::*;
        use diesel::deserialize::QueryableByName;
        use diesel::prelude::*;
        use diesel::row::NamedRow;
        use diesel::sql_query;
        use diesel::sql_types::Text;

        impl QueryableByName<Mysql> for Test {
            fn build<'a>(row: &impl NamedRow<'a, Mysql>) -> deserialize::Result<Self> {
                let value = NamedRow::get(row, "value")?;
                Ok(Test { value })
            }
        }

        impl QueryableByName<Mysql> for NullableTest {
            fn build<'a>(row: &impl NamedRow<'a, Mysql>) -> deserialize::Result<Self> {
                let value = NamedRow::get(row, "value")?;
                Ok(NullableTest { value })
            }
        }

        #[test]
        fn test_null() {
            let mut connection = diesel::MysqlConnection::establish(&get_mysql_url()).expect("Establish connection");

            // Test NULL
            let items: Vec<NullableTest> = sql_query("SELECT CAST(NULL AS DECIMAL) AS value")
                .load(&mut connection)
                .expect("Unable to query value");
            let result = items.first().unwrap().value;
            assert_eq!(None, result);
        }

        #[test]
        fn read_numeric_type() {
            let mut connection = diesel::MysqlConnection::establish(&get_mysql_url()).expect("Establish connection");
            for &(precision, scale, sent, expected) in TEST_DECIMALS.iter() {
                let items: Vec<Test> = sql_query(format!(
                    "SELECT CAST('{}' AS DECIMAL({}, {})) AS value",
                    sent, precision, scale
                ))
                .load(&mut connection)
                .expect("Unable to query value");
                assert_eq!(
                    expected,
                    items.first().unwrap().value.to_string(),
                    "DECIMAL({}, {}) sent: {}",
                    precision,
                    scale,
                    sent
                );
            }
        }

        #[test]
        fn read_native_numeric_types() {
            let mut connection = diesel::MysqlConnection::establish(&get_mysql_url()).expect("Establish connection");
            sql_query(
                "CREATE TEMPORARY TABLE native_numeric_values (
                    tiny_signed TINYINT,
                    tiny_unsigned TINYINT UNSIGNED,
                    small_signed SMALLINT,
                    small_unsigned SMALLINT UNSIGNED,
                    medium_signed MEDIUMINT,
                    medium_unsigned MEDIUMINT UNSIGNED,
                    int_signed INT,
                    int_unsigned INT UNSIGNED,
                    big_signed BIGINT,
                    big_unsigned BIGINT UNSIGNED,
                    float_value FLOAT,
                    double_value DOUBLE,
                    amount DECIMAL(35, 6)
                )",
            )
            .execute(&mut connection)
            .expect("Unable to create native numeric fixture");
            sql_query(
                "INSERT INTO native_numeric_values VALUES (
                    -42, 200, -4200, 60000, -420000, 16000000,
                    825307441, 4000000000, -9000000000000000000,
                    18446744073709551615, 1.5, 1.5, 3950.123456
                )",
            )
            .execute(&mut connection)
            .expect("Unable to insert native numeric fixture");

            let checks = [
                ("int_signed", "825307441"),
                ("tiny_signed", "-42"),
                ("tiny_unsigned", "200"),
                ("small_signed", "-4200"),
                ("small_unsigned", "60000"),
                ("medium_signed", "-420000"),
                ("medium_unsigned", "16000000"),
                ("int_unsigned", "4000000000"),
                ("big_signed", "-9000000000000000000"),
                ("big_unsigned", "18446744073709551615"),
                ("float_value", "1.5"),
                ("double_value", "1.5"),
                ("amount", "3950.123456"),
            ];
            for (column, expected) in checks {
                let cast: Vec<Test> = sql_query(format!(
                    "SELECT CAST({column} AS DECIMAL(35, 6)) AS value FROM native_numeric_values"
                ))
                .load(&mut connection)
                .expect("Unable to read DECIMAL numeric control");
                let expected = Decimal::from_str(expected).unwrap();
                assert_eq!(expected, cast[0].value, "DECIMAL cast of {column}");
                let native: Vec<Test> = sql_query(format!("SELECT {column} AS value FROM native_numeric_values"))
                    .load(&mut connection)
                    .expect("Unable to read native numeric value");
                assert_eq!(expected, native[0].value, "native column {column}");
            }

            let count: Vec<Test> = sql_query("SELECT COUNT(*) AS value FROM native_numeric_values")
                .load(&mut connection)
                .expect("Unable to read COUNT result");
            assert_eq!(Decimal::ONE, count[0].value);
            let count_cast: Vec<Test> =
                sql_query("SELECT CAST(COUNT(*) AS DECIMAL(35, 6)) AS value FROM native_numeric_values")
                    .load(&mut connection)
                    .expect("Unable to read COUNT DECIMAL control");
            assert_eq!(Decimal::ONE, count_cast[0].value);
            let sum: Vec<Test> = sql_query("SELECT SUM(double_value) AS value FROM native_numeric_values")
                .load(&mut connection)
                .expect("Unable to read SUM result");
            assert_eq!(Decimal::from_str("1.5").unwrap(), sum[0].value);
            let sum_cast: Vec<Test> =
                sql_query("SELECT CAST(SUM(double_value) AS DECIMAL(35, 6)) AS value FROM native_numeric_values")
                    .load(&mut connection)
                    .expect("Unable to read SUM DECIMAL control");
            assert_eq!(Decimal::from_str("1.5").unwrap(), sum_cast[0].value);

            let unsupported: Result<Vec<Test>, _> =
                sql_query("SELECT CAST('1' AS CHAR) AS value").load(&mut connection);
            assert!(unsupported.is_err());
        }

        #[test]
        fn write_numeric_type() {
            let mut connection = diesel::MysqlConnection::establish(&get_mysql_url()).expect("Establish connection");
            for &(precision, scale, sent, expected) in TEST_DECIMALS.iter() {
                let items: Vec<Test> =
                    sql_query(format!("SELECT CAST(? AS DECIMAL({}, {})) AS value", precision, scale))
                        .bind::<Text, _>(sent)
                        .load(&mut connection)
                        .expect("Unable to query value");
                assert_eq!(
                    expected,
                    items.first().unwrap().value.to_string(),
                    "DECIMAL({}, {}) sent: {}",
                    precision,
                    scale,
                    sent
                );
            }
        }
    }
}
