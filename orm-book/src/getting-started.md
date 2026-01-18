# Getting Started

## 1. Add Dependencies

```toml
[dependencies]
premix-orm = "1.0.4"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

Enable `postgres` or `mysql` features on both `premix-orm` and `sqlx` if you
plan to use those databases.

```toml
premix-orm = { version = "1.0.4", features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
```

## 2. Define a Model

```rust
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}
```

## 3. Connect and Sync

```rust
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

let mut user = User { id: 0, name: "Alice".to_string() };
user.save(&pool).await?;
# Ok(())
# }
```

Premix creates the table if it does not exist and keeps the SQL simple and
predictable.

## Common Pitfalls

- Enable database features on both `premix-orm` and `sqlx`.
- The CLI uses `DATABASE_URL` by default; pass `--database` if needed.
- `premix migrate down` is not implemented yet.
