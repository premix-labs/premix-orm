# Multi-Database

Premix supports SQLite, Postgres, and MySQL through `sqlx`.

## Feature Flags

SQLite is enabled by default. Enable other databases explicitly:

```toml
premix-orm = { version = "1.0.8-alpha", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
```

For MySQL:

```toml
premix-orm = { version = "1.0.8-alpha", features = ["mysql", "axum", "metrics"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "mysql"] }
```

## Pool Types

Choose the pool type based on your database:

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let database_url = "postgres://localhost/app";
let sqlite = Premix::smart_sqlite_pool("sqlite:app.db").await?;
let pg = Premix::smart_postgres_pool(&database_url).await?;
let mysql = premix_orm::sqlx::MySqlPool::connect(&database_url).await?;
# Ok(())
# }
```

## Smart Pool Configuration

Premix can auto-tune pool settings for server vs serverless environments:

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let sqlite = Premix::smart_sqlite_pool("sqlite:app.db").await?;
let pg = Premix::smart_postgres_pool("postgres://localhost/app").await?;
# Ok(())
# }
```

Set `PREMIX_ENV=serverless` to force serverless tuning. The default profile is
`Server` when no environment hints are present.

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
