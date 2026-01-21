#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! # Premix ORM
//!
//! > **"Write Rust, Run Optimized SQL."**
//!
//! Premix is a zero-overhead, type-safe ORM for Rust. It generates SQL at
//! compile time with macros and executes via `sqlx`.
//!
//! ## Key Features
//!
//! - Auto-sync schema from models.
//! - Compile-time SQL generation (no runtime reflection).
//! - Application-level joins with batched `WHERE IN` queries.
//! - SQLite, Postgres, and MySQL support via `sqlx`.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use premix_orm::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Model, Debug, Serialize, Deserialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
//! premix_orm::Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
//!
//! let mut user = User { id: 0, name: "Alice".to_string() };
//! user.save(&pool).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Relations and Eager Loading
//!
//! ```rust,no_run
//! use premix_orm::prelude::*;
//!
//! #[derive(Model)]
//! struct User {
//!     id: i32,
//!     name: String,
//!
//!     #[has_many(Post)]
//!     #[premix(ignore)]
//!     posts: Option<Vec<Post>>,
//! }
//!
//! #[derive(Model)]
//! #[belongs_to(User)]
//! struct Post {
//!     id: i32,
//!     user_id: i32,
//!     title: String,
//! }
//!
//! # async fn example(pool: premix_orm::sqlx::SqlitePool) -> Result<(), sqlx::Error> {
//! let _users = User::find_in_pool(&pool).include("posts").all().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Bulk Operations
//!
//! ```rust,no_run
//! use premix_orm::prelude::*;
//! use serde_json::json;
//!
//! #[derive(Model)]
//! struct User {
//!     id: i32,
//!     status: String,
//! }
//!
//! # async fn example(pool: premix_orm::sqlx::SqlitePool) -> Result<(), sqlx::Error> {
//! let _updated = User::find_in_pool(&pool)
//!     .filter_eq("status", "inactive")
//!     .update(json!({ "status": "active" }))
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! premix-orm = "1.0.7-alpha"
//! ```
//!
//! ## Book
//!
//! A longer-form guide lives in `orm-book/` at the repository root. It covers
//! models, queries, relations, migrations, transactions, and limitations.

pub use premix_core::*;
pub use premix_macros::Model;
/// Compile-time query macro for true Zero-Overhead SQL generation.
///
/// This macro generates SQL at compile time, achieving 0% overhead compared to raw sqlx.
///
/// # Example
///
/// ```ignore
/// use premix_orm::prelude::*;
///
/// let user = premix_query!(User, SELECT, filter_eq("id", user_id), limit(1))
///     .fetch_one(&pool)
///     .await?;
/// ```
pub use premix_macros::premix_query;

/// Integration with common web frameworks (Axum, Actix, etc.).
pub mod integrations;

#[cfg(feature = "axum")]
pub use integrations::axum::*;

#[cfg(feature = "actix")]
pub use integrations::actix::*;

/// The Premix prelude, re-exporting commonly used traits and types.
pub mod prelude {
    pub use premix_core::prelude::*;

    pub use crate::Model; // The macro
    pub use crate::premix_query; // Zero-overhead compile-time query macro
}
