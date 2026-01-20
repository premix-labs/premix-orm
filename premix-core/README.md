# Premix Core

The foundational library for **Premix ORM**.

`premix-core` provides the essential traits, types, and logic that power the ORM functionality. It includes:
- Database abstraction traits (`SqlDialect`)
- Executor logic (`IntoExecutor`, `Executor`)
- Model traits (`Model`)
- SQLx integration helpers

## Research Status

This crate is part of a research prototype. APIs may change and production use is not recommended yet.

## Installation

Most users should depend on `premix-orm` instead. Use `premix-core` directly only if you are building extensions or low-level tooling.

```toml
[dependencies]
premix-core = "1.0.6-alpha"
```

## Quick Start

```rust
use premix_core::{Model, Premix};
use sqlx::SqlitePool;

#[derive(Model, Debug)]
struct User {
    id: i32,
    name: String,
}

async fn example() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;
    Ok(())
}
```

## Features

- **sqlite** (default): Support for SQLite
- **postgres**: Support for PostgreSQL
- **mysql**: Support for MySQL (partial)

## Compatibility

`premix-core` uses `sqlx` under the hood. Make sure your `sqlx` features match the database feature you enable here.

## License

This project is licensed under the [MIT license](LICENSE).

