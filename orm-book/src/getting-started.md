# Getting Started

This chapter walks through a minimal but complete setup: dependencies, model
definition, connection, schema sync, and a simple query.

## 1. Add Dependencies

```toml
[dependencies]
premix-orm = "1.0.5"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Database Feature Flags

Enable the database features you need on both `premix-orm` and `sqlx`:

```toml
premix-orm = { version = "1.0.5", features = ["postgres"] }
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
let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Ok(())
# }
```

For Postgres:

```rust,no_run
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let database_url = "postgres://localhost/app";
let pool = premix_orm::sqlx::PgPool::connect(&database_url).await?;
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
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
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
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let mut user = User { id: 0, name: "Alice".to_string() };
user.save(&pool).await?;

let users = User::find_in_pool(&pool)
    .filter("name = 'Alice'")
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

- Enable database features on both `premix-orm` and `sqlx`.
- The CLI reads `DATABASE_URL` or defaults to `sqlite:premix.db`.
- `premix migrate down` reverts the most recent migration.
- `filter()` accepts raw SQL strings; use carefully to avoid injection.

