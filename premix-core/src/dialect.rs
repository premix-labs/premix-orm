use sqlx::Database;

// Chapter 18: Multi-Database Support
// We define a trait that encapsulates all the requirements for a database to work with Premix.
pub trait SqlDialect: Database + Sized + Send + Sync
where
    Self::Connection: Send,
{
    fn placeholder(n: usize) -> String;
    fn auto_increment_pk() -> &'static str;
    fn rows_affected(res: &Self::QueryResult) -> u64;
    fn last_insert_id(res: &Self::QueryResult) -> i64;
    fn supports_returning() -> bool {
        false
    }

    fn current_timestamp_fn() -> &'static str {
        "CURRENT_TIMESTAMP"
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
        "REAL"
    }
    fn blob_type() -> &'static str {
        "BLOB"
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
