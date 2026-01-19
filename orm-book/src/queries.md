# Queries

Premix uses a minimal query builder focused on simple, fast queries. The API
is intentionally small so it stays predictable and easy to reason about.

## Find by ID

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
let user = User::find_by_id(&pool, 1).await?;
# Ok(())
# }
```

If the row does not exist, the result is `Ok(None)`.

## Query Builder Basics

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let users = User::find_in_pool(&pool)
    .filter("age > 18")
    .limit(20)
    .offset(0)
    .all()
    .await?;
# Ok(())
# }
```

### Supported Methods

- `filter("...")`: Adds raw SQL to the `WHERE` clause.
- `limit(n)` / `offset(n)`: Pagination.
- `include("relation")`: Eager-load relations (see Relations chapter).
- `with_deleted()`: Include soft-deleted rows.
- `all()`: Execute and return `Vec<Model>`.
- `update(json)`: Bulk update.
- `delete()`: Bulk delete or soft delete.

## Filters and Safety

`filter()` accepts raw SQL fragments. This keeps the builder fast and small,
but it also means **you are responsible for SQL safety**. Prefer:

- Static filters where values are trusted.
- Raw SQL queries for parameter binding when values are user-provided.

For complex conditions, use the raw SQL escape hatch:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let rows = User::raw_sql("SELECT * FROM users WHERE age > 18")
    .fetch_all(&pool)
    .await?;
# Ok(())
# }
```

## Pagination and Soft Deletes

When `deleted_at` is present on a model:

- Default queries exclude deleted rows.
- `with_deleted()` includes them.

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    deleted_at: Option<String>,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let users = User::find_in_pool(&pool)
    .with_deleted()
    .limit(100)
    .all()
    .await?;
# Ok(())
# }
```

## Bulk Update and Delete

Bulk operations are detailed in the Bulk Ops chapter, but the key constraints
are:

- An `update()` or `delete()` without a filter is rejected.
- `update()` expects a JSON object payload.
- `delete()` soft-deletes when `deleted_at` exists.
