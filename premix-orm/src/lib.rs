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
//! let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
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
//!     .filter("status = 'inactive'")
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
//! premix-orm = "1.0.3"
//! ```
//!
//! ## Book
//!
//! A longer-form guide lives in `orm-book/` at the repository root. It covers
//! models, queries, relations, migrations, transactions, and limitations.

pub use premix_core::*;
pub use premix_macros::Model;

pub mod prelude {
    pub use premix_core::prelude::*;

    pub use crate::Model; // The macro // The traits and other core items
}
