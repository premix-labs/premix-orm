# Multi-Database

Premix supports SQLite, Postgres, and MySQL through `sqlx`.

## Feature Flags

SQLite is enabled by default. Enable other databases explicitly:

```toml
premix-orm = { version = "1.0.4", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
```

For MySQL:

```toml
premix-orm = { version = "1.0.4", features = ["mysql"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "mysql"] }
```

## Pool Types

Choose the pool type based on your database:

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let database_url = "postgres://localhost/app";
let sqlite = premix_orm::sqlx::SqlitePool::connect("sqlite:app.db").await?;
let pg = premix_orm::sqlx::PgPool::connect(&database_url).await?;
let mysql = premix_orm::sqlx::MySqlPool::connect(&database_url).await?;
# Ok(())
# }
```

## SqlDialect

`SqlDialect` abstracts database-specific behavior:

- Placeholder style (`?` for SQLite/MySQL, `$1` for Postgres).
- Auto-increment primary key syntax.
- Last insert ID semantics.

This allows `Model<DB>` and `QueryBuilder<DB>` to work across databases.

## Important Notes

- Enable the same database features on both `premix-orm` and `sqlx`.
- The book's executable examples are validated with SQLite and Postgres. MySQL
  support is feature-gated but not covered by the book examples yet.
- SQLite and Postgres differ in type semantics; prefer explicit migrations for
  production schemas.
