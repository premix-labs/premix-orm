# Getting Started

> ⚠️ **Note:** Premix is an AI-assisted research prototype. APIs may change and
> production use is not recommended. See [DISCLAIMER](../../DISCLAIMER.md).

This chapter walks through a minimal but complete setup: dependencies, model
definition, connection, schema sync, and a simple query.

## Requirements

- Rust 1.85+ (edition 2024).
- No nightly toolchain required.

## 1. Add Dependencies

```toml
[dependencies]
premix-orm = "1.0.7-alpha"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Database Feature Flags

Enable the database features you need on both `premix-orm` and `sqlx`:

```toml
premix-orm = { version = "1.0.7-alpha", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
```

If you use MySQL, swap `postgres` for `mysql`. SQLite is enabled by default.

## 2. Define a Model

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}
```

`#[derive(Model)]` generates schema metadata and query helpers.

## 3. Connect to the Database

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Ok(())
# }
```

For Postgres:

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let database_url = "postgres://localhost/app";
let pool = Premix::smart_postgres_pool(&database_url).await?;
# Ok(())
# }
```

## 4. Create or Sync the Schema

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
# Ok(())
# }
```

`Premix::sync` auto-creates the table from your model definition. This is
excellent for prototyping and tests. For production, use versioned SQL
migrations instead.

## 5. Insert and Query

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let mut user = User { id: 0, name: "Alice".to_string() };
user.save(&pool).await?;

let users = User::find_in_pool(&pool)
    .filter_eq("name", "Alice")
    .all()
    .await?;
# Ok(())
# }
```

## 6. Optional: CLI Migrations

Install the CLI:

```bash
cargo install premix-cli
```

Create a migration:

```bash
premix migrate create create_users
```

Apply migrations:

```bash
premix migrate up
```

## Common Pitfalls

- Enable database and integration features (e.g., `axum`, `actix`) on `premix-orm`.
- The CLI reads `DATABASE_URL` or defaults to `sqlite:premix.db`.
- `premix migrate down` reverts the most recent migration.
- `filter()`/`filter_raw()` accept raw SQL strings and require `.allow_unsafe()`; use carefully to avoid injection.
