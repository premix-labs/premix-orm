//! # Premix ORM 🚀
//!
//! > **"Write Rust, Run Optimized SQL."**
//!
//! Premix is a **Zero-Overhead, Type-Safe ORM** for Rust that eliminates the need for manual migration files.
//! It combines the ease of use of Active Record with the raw performance of handcrafted SQL.
//!
//! ## 🌟 Key Features
//!
//! - **🪄 Auto-Sync Schema**: Syncs your Rust structs directly to the database.
//! - **⚡ Zero Overhead**: Uses macros to generate SQL at compile-time.
//! - **🚀 Application-Level Joins**: Solves N+1 problems with smart `WHERE IN` queries.
//! - **🌍 Multi-Database**: Support for SQLite, Postgres, and MySQL.
//!
//! ## 🚀 Quick Start
//!
//! ```rust,no_run
//! use premix_orm::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Debug, Serialize, Deserialize)]
//! struct User {
//!     id: i32,
//!     name: String,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect and Sync
//! let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
//! premix_orm::Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
//!
//! // Create
//! let mut user = User { id: 0, name: "Alice".to_string() };
//! user.save(&pool).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 📦 Installation
//!
//! ```toml
//! [dependencies]
//! premix-orm = "1.0.2"
//! ```

pub use premix_core::*;
pub use premix_macros::Model;

pub mod prelude {
    pub use premix_core::prelude::*;

    pub use crate::Model; // The macro // The traits and other core items
}
