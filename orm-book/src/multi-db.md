# Multi-Database

Premix supports SQLite, Postgres, and MySQL through `sqlx`.

- SQLite: enabled by default.
- Postgres: enable the `postgres` feature on `sqlx`.
- MySQL: enable the `mysql` feature on `sqlx`.

Example `Cargo.toml`:

```toml
premix-orm = { version = "1.0.4", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
```

At runtime, you choose the pool type:

```rust
let pool = premix_orm::sqlx::PgPool::connect(&database_url).await?;
```

The `SqlDialect` trait abstracts placeholders and dialect-specific behavior.

Note: `premix-orm` features forward to `premix-core` features. Enable the
database you need on both `premix-orm` and `sqlx`.
