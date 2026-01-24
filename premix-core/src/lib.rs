#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![warn(rust_2018_idioms)]

//! # Premix ORM Core
//!
//! Core definitions and traits for the Premix ORM ecosystem.
//! Provides the `Model` trait, `Executor` abstraction, and `QueryBuilder`.
//!
//! ## Architecture
//!
//! - **Model**: The core trait that all database entities must implement.
//! - **Executor**: Abstracts over connection pools and transactions.
//! - **QueryBuilder**: A type-safe SQL query generator with support for:
//!   - Filtering (eq, ne, gt, lt, in)
//!   - Pagination (limit, offset)
//!   - Relations (eager loading)
//!   - Soft Deletes
//!
//! This crate is designed to be used with `premix-macros` for compile-time SQL generation overrides.

// Re-export common types
pub use chrono;
pub use sqlx;
pub use tracing;
pub use uuid;

// New Modules
/// SQL dialect abstractions for multi-database support.
pub mod dialect;
/// Database executor abstraction for connection pools and transactions.
pub mod executor;
/// Database migration engine.
pub mod migrator;
pub use migrator::{Migration, Migrator};
/// Metrics and monitoring.
#[cfg(feature = "metrics")]
pub mod metrics;
/// Core traits and types for database models.
pub mod model;
pub use model::{Model, ModelHooks, ModelValidation, UpdateResult, ValidationError};
/// Type-safe SQL query builder.
pub mod query;
pub use query::QueryBuilder;
/// Database schema introspection and diffing utilities.
pub mod schema;
pub use schema::ModelSchema;
/// Cache helpers for SQL snippets/placeholders.
pub mod sql_cache;
pub use sql_cache::cached_placeholders;

/// Main entry point for the Premix ORM helpers.
#[derive(Debug, Clone, Copy, Default)]
pub struct Premix;

impl Premix {
    /// Creates a smart SQLite pool with optimized settings for performance.
    #[cfg(feature = "sqlite")]
    pub async fn smart_sqlite_pool(url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
        use std::str::FromStr;
        let options = SqliteConnectOptions::from_str(url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        sqlx::SqlitePool::connect_with(options).await
    }

    /// Synchronizes the database schema for a specific model.
    pub async fn sync<DB, T>(pool: &sqlx::Pool<DB>) -> Result<(), sqlx::Error>
    where
        DB: crate::dialect::SqlDialect,
        T: crate::model::Model<DB> + crate::schema::ModelSchema,
        for<'c> &'c sqlx::Pool<DB>: sqlx::Executor<'c, Database = DB>,
    {
        let schema = T::schema();
        let sql = schema.to_create_sql();
        use sqlx::Executor;
        pool.execute(sql.as_str()).await?;
        Ok(())
    }
}

/// Helper to build a comma-separated list of placeholders for a given database.
pub fn build_placeholders<DB: crate::dialect::SqlDialect>(start: usize, count: usize) -> String {
    (start..start + count)
        .map(DB::placeholder)
        .collect::<Vec<_>>()
        .join(", ")
}

pub use dialect::SqlDialect;
pub use executor::{Executor, IntoExecutor};

// Prelude
/// The Premix prelude, re-exporting commonly used traits and types.
pub mod prelude {
    pub use crate::Premix;
    pub use crate::build_placeholders;
    pub use crate::dialect::SqlDialect;
    pub use crate::executor::{Executor, IntoExecutor};
    pub use crate::migrator::{Migration, Migrator};
    pub use crate::model::{Model, ModelHooks, ModelValidation, UpdateResult, ValidationError};
    pub use crate::query::QueryBuilder;
    pub use crate::schema::ModelSchema;
    pub use crate::sql_cache::cached_placeholders;
}
