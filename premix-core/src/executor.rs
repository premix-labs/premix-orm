use crate::dialect::SqlDialect;
use sqlx::Database;

// Chapter 7: Stronger Executor Abstraction
pub enum Executor<'a, DB: Database> {
    Pool(&'a sqlx::Pool<DB>),
    Conn(&'a mut DB::Connection),
}

unsafe impl<'a, DB: Database> Send for Executor<'a, DB> where DB::Connection: Send {}
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

pub trait IntoExecutor<'a>: Send + 'a {
    type DB: SqlDialect;
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
}
