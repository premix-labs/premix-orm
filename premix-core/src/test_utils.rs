use std::future::Future;
use std::pin::Pin;

use sqlx::Database;

/// Run an async test block inside a transaction that is always rolled back.
pub async fn with_test_transaction<DB, F, T>(pool: &sqlx::Pool<DB>, f: F) -> Result<T, sqlx::Error>
where
    DB: Database,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    F: for<'c> FnOnce(
        &'c mut <DB as Database>::Connection,
    ) -> Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + 'c>>,
{
    let mut tx = pool.begin().await?;
    let result = f(tx.as_mut()).await;
    let rollback_result = tx.rollback().await;

    match (result, rollback_result) {
        (Ok(value), Ok(_)) => Ok(value),
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
    }
}

/// Lightweight test helper for constructing a dedicated pool.
pub struct MockDatabase<DB: Database> {
    pool: sqlx::Pool<DB>,
}

impl<DB: Database> MockDatabase<DB> {
    pub fn pool(&self) -> &sqlx::Pool<DB> {
        &self.pool
    }

    pub fn into_pool(self) -> sqlx::Pool<DB> {
        self.pool
    }
}

#[cfg(feature = "sqlite")]
impl MockDatabase<sqlx::Sqlite> {
    pub async fn new_sqlite() -> Result<Self, sqlx::Error> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Ok(Self { pool })
    }
}
