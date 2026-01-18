# Getting Started

## 1. Add Dependencies

```toml
[dependencies]
premix-orm = "1.0.3"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

Enable `postgres` or `mysql` features on `sqlx` if you plan to use those
databases.

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
