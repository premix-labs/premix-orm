use crate::dialect::SqlDialect;
use sqlx::Database;

// Chapter 7: Stronger Executor Abstraction
/// A unified database executor that can wrap either a connection pool or a single connection.
///
/// This allows Premix to remain agnostic about whether it's executing within a transaction,
/// a shared pool, or a dedicated connection.
pub enum Executor<'a, DB: Database> {
    /// A shared connection pool.
    Pool(&'a sqlx::Pool<DB>),
    /// A single, mutable database connection.
    Conn(&'a mut DB::Connection),
}

impl<'a, DB: Database> std::fmt::Debug for Executor<'a, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pool(_) => f.write_str("Executor::Pool"),
            Self::Conn(_) => f.write_str("Executor::Conn"),
        }
    }
}

// SAFETY: Executor only contains a reference to a Pool or a mutable reference to a Connection.
// Pools are thread-safe by design in sqlx. Connections are usually Send, and our bounds
// ensure that the manual implementation matches the underlying database driver's capabilities.
unsafe impl<'a, DB: Database> Send for Executor<'a, DB> where DB::Connection: Send {}
// SAFETY: Similarly, Executor is Sync if the underlying Connection/Pool is Sync.
unsafe impl<'a, DB: Database> Sync for Executor<'a, DB> where DB::Connection: Sync {}

impl<'a, DB: Database> From<&'a sqlx::Pool<DB>> for Executor<'a, DB> {
    fn from(pool: &'a sqlx::Pool<DB>) -> Self {
        Self::Pool(pool)
    }
}

impl<'a, DB: Database> From<&'a mut DB::Connection> for Executor<'a, DB> {
    fn from(conn: &'a mut DB::Connection) -> Self {
        Self::Conn(conn)
    }
}

/// A trait for types that can be converted into an [`Executor`].
pub trait IntoExecutor<'a>: Send + 'a {
    /// The database dialect associated with this executor.
    type DB: SqlDialect;
    /// Converts the type into an [`Executor`].
    fn into_executor(self) -> Executor<'a, Self::DB>;
}

impl<'a, DB: SqlDialect> IntoExecutor<'a> for &'a sqlx::Pool<DB> {
    type DB = DB;
    fn into_executor(self) -> Executor<'a, DB> {
        Executor::Pool(self)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> IntoExecutor<'a> for &'a mut sqlx::SqliteConnection {
    type DB = sqlx::Sqlite;
    fn into_executor(self) -> Executor<'a, Self::DB> {
        Executor::Conn(self)
    }
}

#[cfg(feature = "postgres")]
impl<'a> IntoExecutor<'a> for &'a mut sqlx::postgres::PgConnection {
    type DB = sqlx::Postgres;
    fn into_executor(self) -> Executor<'a, Self::DB> {
        Executor::Conn(self)
    }
}

impl<'a, DB: SqlDialect> IntoExecutor<'a> for Executor<'a, DB> {
    type DB = DB;
    fn into_executor(self) -> Executor<'a, DB> {
        self
    }
}

impl<'a, DB: Database> Executor<'a, DB> {
    /// Executes a SQL query and returns the database result (e.g., number of rows affected).
    pub async fn execute<'q, A>(
        &mut self,
        query: sqlx::query::Query<'q, DB, A>,
    ) -> Result<DB::QueryResult, sqlx::Error>
    where
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.execute(*pool).await,
            Self::Conn(conn) => query.execute(&mut **conn).await,
        }
    }

    /// Executes a SQL query and fetches all resulting rows.
    pub async fn fetch_all<'q, T, A>(
        &mut self,
        query: sqlx::query::QueryAs<'q, DB, T, A>,
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.fetch_all(*pool).await,
            Self::Conn(conn) => query.fetch_all(&mut **conn).await,
        }
    }

    /// Executes a SQL query and fetches an optional single row.
    pub async fn fetch_optional<'q, T, A>(
        &mut self,
        query: sqlx::query::QueryAs<'q, DB, T, A>,
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin,
        A: sqlx::IntoArguments<'q, DB> + 'q,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.fetch_optional(*pool).await,
            Self::Conn(conn) => query.fetch_optional(&mut **conn).await,
        }
    }

    /// Executes a SQL query and returns a stream of resulting rows.
    pub fn fetch_stream<'q, T, A>(
        self,
        query: sqlx::query::QueryAs<'q, DB, T, A>,
    ) -> futures_util::stream::BoxStream<'a, Result<T, sqlx::Error>>
    where
        T: for<'r> sqlx::FromRow<'r, DB::Row> + Send + Unpin + 'a,
        A: sqlx::IntoArguments<'q, DB> + 'q,
        'q: 'a,
        DB: SqlDialect,
        for<'c> &'c mut DB::Connection: sqlx::Executor<'c, Database = DB>,
    {
        match self {
            Self::Pool(pool) => query.fetch(pool),
            Self::Conn(conn) => query.fetch(conn),
        }
    }
}
