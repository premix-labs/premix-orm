# Premix ORM

**Premix ORM** is a zero-overhead, type-safe ORM for Rust, designed for performance and developer experience.

This crate (`premix`) is a **facade** that re-exports the core logic and macros, providing a unified entry point for your application.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
premix-orm = "1.0.0"
```

## Quick Start

```rust
use premix::Model;
use premix::prelude::*; // If we add a prelude later

#[derive(Model, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
}

// ... usage
let user = User::find_by_id(executor, 1).await?;
```

## Features

- **Type-Safe Queries**: Leverages `sqlx` for compile-time checked SQL execution.
- **Auto-Derive**: `#[derive(Model)]` handles all the boilerplate.
- **Performance**: Zero-overhead abstraction over raw SQL queries.
- **Multi-Database**: Support for SQLite, Postgres, and MySQL.

## License

This project is licensed under the [MIT license](LICENSE).
