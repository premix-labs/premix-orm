# Premix ORM 🚀

**Premix ORM** is a zero-overhead, type-safe ORM for Rust, designed for performance and developer experience.

This crate (`premix-orm`) is the **official facade** that re-exports `premix-core` and `premix-macros`, providing a unified, unambiguous entry point for your application.

## 🌟 Why use this Facade?

Previously, users had to manage both `premix-core` and `premix-macros`. With `premix-orm`, you get:
1.  **Unified Imports**: `use premix_orm::prelude::*;` gets you everything.
2.  **No Version Mismatch**: We ensure the core and macros versions are always compatible.
3.  **Clean Dependencies**: Only one crate to add to your `Cargo.toml`.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
premix-orm = "1.0.2"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

## 🚀 Quick Start

```rust
use premix_orm::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Model, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    
    // Soft Delete is auto-detected by field name
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Sync schema
    Premix::sync::<User, _>(&pool).await?;
    
    // CRUD
    let mut user = User { id: 0, name: "Alice".to_string(), deleted_at: None };
    user.save(&pool).await?;
    
    println!("Saved user with ID: {}", user.id);
    Ok(())
}
```

## 📄 License

This project is licensed under the [MIT license](LICENSE).
