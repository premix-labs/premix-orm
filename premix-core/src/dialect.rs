use sqlx::Database;

// Chapter 18: Multi-Database Support
// We define a trait that encapsulates all the requirements for a database to work with Premix.
/// A trait that encapsulates all the requirements for a database to work with Premix.
///
/// Implementing this trait allows Premix to generate correct SQL syntax and handle
/// database-specific behaviors like placeholder styles and identifier quoting.
pub trait SqlDialect: Database + Sized + Send + Sync
where
    Self::Connection: Send,
{
    /// Returns the placeholder for the `n`-th parameter in a query (e.g., "?" or "$1").
    fn placeholder(n: usize) -> String;
    /// Returns the SQL fragment for an auto-incrementing Primary Key.
    fn auto_increment_pk() -> &'static str;
    /// Returns the number of rows affected by a query result.
    fn rows_affected(res: &Self::QueryResult) -> u64;
    /// Returns the ID of the last inserted row.
    fn last_insert_id(res: &Self::QueryResult) -> i64;
    /// Returns true if the database supports the `RETURNING` clause.
    fn supports_returning() -> bool {
        false
    }

    /// Returns the SQL function code for getting the current timestamp.
    fn current_timestamp_fn() -> &'static str {
        "CURRENT_TIMESTAMP"
    }
    /// Returns the native SQL type for 32-bit integers.
    fn int_type() -> &'static str {
        "INTEGER"
    }
    /// Returns the native SQL type for 64-bit integers.
    fn bigint_type() -> &'static str {
        "BIGINT"
    }
    /// Returns the native SQL type for text strings.
    fn text_type() -> &'static str {
        "TEXT"
    }
    /// Returns the native SQL type for booleans.
    fn bool_type() -> &'static str {
        "BOOLEAN"
    }
    /// Returns the native SQL type for floating-point numbers.
    fn float_type() -> &'static str {
        "REAL"
    }
    /// Returns the native SQL type for binary data.
    fn blob_type() -> &'static str {
        "BLOB"
    }

    /// Quotes an identifier (table/column name) to prevent SQL injection.
    fn quote_identifier(ident: &str) -> String {
        format!("`{}`", ident.replace('`', "``"))
    }
}

#[cfg(feature = "sqlite")]
impl SqlDialect for sqlx::Sqlite {
    fn placeholder(_n: usize) -> String {
        "?".to_string()
    }
    fn auto_increment_pk() -> &'static str {
        "INTEGER PRIMARY KEY"
    }
    fn bigint_type() -> &'static str {
        "INTEGER"
    }
    fn blob_type() -> &'static str {
        "BLOB"
    }
    fn rows_affected(res: &sqlx::sqlite::SqliteQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(res: &sqlx::sqlite::SqliteQueryResult) -> i64 {
        res.last_insert_rowid()
    }
    fn supports_returning() -> bool {
        false
    }
}

#[cfg(feature = "postgres")]
impl SqlDialect for sqlx::Postgres {
    fn placeholder(n: usize) -> String {
        format!("${}", n)
    }
    fn auto_increment_pk() -> &'static str {
        "SERIAL PRIMARY KEY"
    }
    fn int_type() -> &'static str {
        "INTEGER"
    }
    fn bigint_type() -> &'static str {
        "BIGINT"
    }
    fn text_type() -> &'static str {
        "TEXT"
    }
    fn bool_type() -> &'static str {
        "BOOLEAN"
    }
    fn float_type() -> &'static str {
        "DOUBLE PRECISION"
    }
    fn blob_type() -> &'static str {
        "BYTEA"
    }
    fn rows_affected(res: &sqlx::postgres::PgQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(_res: &sqlx::postgres::PgQueryResult) -> i64 {
        0
    }
    fn supports_returning() -> bool {
        true
    }
    fn quote_identifier(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }
}

#[cfg(feature = "mysql")]
impl SqlDialect for sqlx::MySql {
    fn placeholder(_n: usize) -> String {
        "?".to_string()
    }
    fn auto_increment_pk() -> &'static str {
        "INTEGER AUTO_INCREMENT PRIMARY KEY"
    }
    fn bigint_type() -> &'static str {
        "BIGINT"
    }
    fn blob_type() -> &'static str {
        "LONGBLOB"
    }
    fn rows_affected(res: &sqlx::mysql::MySqlQueryResult) -> u64 {
        res.rows_affected()
    }
    fn last_insert_id(res: &sqlx::mysql::MySqlQueryResult) -> i64 {
        res.last_insert_id() as i64
    }
    fn supports_returning() -> bool {
        false
    }
}
