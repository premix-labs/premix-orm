// lib.rs
pub use sqlx;
pub use tracing;

pub mod prelude {
    pub use crate::{
        Executor, IntoExecutor, Model, ModelHooks, ModelSchema, ModelValidation, Premix,
        RuntimeProfile, UpdateResult, test_utils::MockDatabase, test_utils::with_test_transaction,
    };
    // Include SqlDialect for generic constraints if users need it often?
    // Usually hidden by Model<DB> but useful for advanced usage.
    pub use crate::dialect::SqlDialect;
}
use sqlx::Database;
// Removed unused imports: Executor, IntoArguments, Duration, Instant
// use sqlx::{Database, Executor as SqlxExecutor, IntoArguments};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeProfile {
    Server,
    Serverless,
}

pub struct Premix;

impl Premix {
    pub fn detect_runtime_profile() -> RuntimeProfile {
        if let Ok(value) = std::env::var("PREMIX_ENV") {
            let value = value.to_ascii_lowercase();
            if value.contains("serverless") || value.contains("lambda") || value.contains("edge") {
                return RuntimeProfile::Serverless;
            }
            if value.contains("server") || value.contains("prod") || value.contains("production") {
                return RuntimeProfile::Server;
            }
        }

        if std::env::var_os("AWS_LAMBDA_FUNCTION_NAME").is_some()
            || std::env::var_os("LAMBDA_TASK_ROOT").is_some()
            || std::env::var_os("K_SERVICE").is_some()
            || std::env::var_os("VERCEL").is_some()
        {
            return RuntimeProfile::Serverless;
        }

        RuntimeProfile::Server
    }

    #[cfg(feature = "sqlite")]
    pub fn sqlite_pool_options(profile: RuntimeProfile) -> sqlx::sqlite::SqlitePoolOptions {
        match profile {
            RuntimeProfile::Server => sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(10)
                .min_connections(1)
                .acquire_timeout(Duration::from_secs(30))
                .idle_timeout(Some(Duration::from_secs(10 * 60)))
                .max_lifetime(Some(Duration::from_secs(30 * 60))),
            RuntimeProfile::Serverless => sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(2)
                .min_connections(0)
                .acquire_timeout(Duration::from_secs(10))
                .idle_timeout(Some(Duration::from_secs(30)))
                .max_lifetime(Some(Duration::from_secs(5 * 60))),
        }
    }

    #[cfg(feature = "postgres")]
    pub fn postgres_pool_options(profile: RuntimeProfile) -> sqlx::postgres::PgPoolOptions {
        match profile {
            RuntimeProfile::Server => sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .min_connections(1)
                .acquire_timeout(Duration::from_secs(30))
                .idle_timeout(Some(Duration::from_secs(10 * 60)))
                .max_lifetime(Some(Duration::from_secs(30 * 60))),
            RuntimeProfile::Serverless => sqlx::postgres::PgPoolOptions::new()
                .max_connections(2)
                .min_connections(0)
                .acquire_timeout(Duration::from_secs(10))
                .idle_timeout(Some(Duration::from_secs(30)))
                .max_lifetime(Some(Duration::from_secs(5 * 60))),
        }
    }

    #[cfg(feature = "sqlite")]
    pub async fn smart_sqlite_pool(database_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
        Self::sqlite_pool_options(Self::detect_runtime_profile())
            .connect(database_url)
            .await
    }

    #[cfg(feature = "sqlite")]
    pub async fn smart_sqlite_pool_with_profile(
        database_url: &str,
        profile: RuntimeProfile,
    ) -> Result<sqlx::SqlitePool, sqlx::Error> {
        Self::sqlite_pool_options(profile)
            .connect(database_url)
            .await
    }

    #[cfg(feature = "postgres")]
    pub async fn smart_postgres_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
        Self::postgres_pool_options(Self::detect_runtime_profile())
            .connect(database_url)
            .await
    }

    #[cfg(feature = "postgres")]
    pub async fn smart_postgres_pool_with_profile(
        database_url: &str,
        profile: RuntimeProfile,
    ) -> Result<sqlx::PgPool, sqlx::Error> {
        Self::postgres_pool_options(profile)
            .connect(database_url)
            .await
    }
}

impl Premix {
    pub async fn sync<DB, M>(pool: &sqlx::Pool<DB>) -> Result<(), sqlx::Error>
    where
        DB: dialect::SqlDialect,
        M: Model<DB>,
        for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
        for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
        for<'c> &'c str: sqlx::ColumnIndex<DB::Row>,
    {
        let sql = M::create_table_sql();
        tracing::debug!(operation = "sync", sql = %sql, "premix sync");
        sqlx::query::<DB>(&sql).execute(pool).await?;
        Ok(())
    }
}

// New Modules
pub mod dialect;
pub mod executor;
pub mod model;
pub mod query;

#[cfg(feature = "metrics")]
pub mod metrics;

// Re-exports to keep API compat where possible or logical
pub use dialect::SqlDialect;
// Executor and IntoExecutor moved to executor module
pub use executor::{Executor, IntoExecutor};
// QueryBuilder moved to query module
pub use query::{BindValue, QueryBuilder}; // BindValue might be needed?
// Model traits moved to model module
pub use model::{Model, ModelHooks, ModelValidation, UpdateResult, ValidationError};

// RawQuery was small, can keep here or move to query. Let's move to query?
// For now, if it wasn't moved, we define it or if it was omitted in query.rs (I missed checking if I copied RawQuery struct).
// I didn't see `pub struct RawQuery` in my query.rs write.
// Let's re-add it here or put it in query.rs. Ideally query.rs.
// I will keep it here for now to avoid compilation error if I missed it, or add to query.rs in a patch if needed.
// Actually, I can just define it here as it's small.
pub struct RawQuery<'q> {
    sql: &'q str,
}

impl Premix {
    pub fn raw<'q>(sql: &'q str) -> RawQuery<'q> {
        RawQuery { sql }
    }
}

impl<'q> RawQuery<'q> {
    pub fn fetch_as<DB, T>(self) -> sqlx::query::QueryAs<'q, DB, T, <DB as Database>::Arguments<'q>>
    where
        DB: Database,
        for<'r> T: sqlx::FromRow<'r, DB::Row>,
    {
        sqlx::query_as::<DB, T>(self.sql)
    }
}

#[doc(hidden)]
pub fn build_placeholders<DB: SqlDialect>(start_index: usize, count: usize) -> String {
    let mut out = String::new();
    for i in 0..count {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&DB::placeholder(start_index + i));
    }
    out
}
pub mod migrator;
pub mod schema;
pub mod test_utils;
pub use migrator::{Migration, Migrator};
pub use schema::{
    ColumnDiff, ColumnNullabilityDiff, ColumnPrimaryKeyDiff, ColumnTypeDiff, ModelSchema,
    SchemaColumn, SchemaDiff, SchemaForeignKey, SchemaIndex, SchemaTable,
    format_schema_diff_summary,
};
#[cfg(feature = "postgres")]
pub use schema::{diff_postgres_schema, introspect_postgres_schema, postgres_migration_sql};
#[cfg(feature = "sqlite")]
pub use schema::{diff_sqlite_schema, introspect_sqlite_schema, sqlite_migration_sql};
pub use test_utils::{MockDatabase, with_test_transaction};
