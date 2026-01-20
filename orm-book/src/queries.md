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
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
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
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let users = User::find_in_pool(&pool)
    .filter_gt("age", 18)
    .limit(20)
    .offset(0)
    .all()
    .await?;
# Ok(())
# }
```

### Supported Methods

- `filter_eq/lt/lte/gt/gte/like/in/is_null/is_not_null(...)`: Safe filters with bound values.
- `filter("...")`: Adds raw SQL to the `WHERE` clause (unsafe for user input; requires `.allow_unsafe()`).
- `filter_raw("...")`: Explicit raw SQL filter (same as `filter`, requires `.allow_unsafe()`).
- `limit(n)` / `offset(n)`: Pagination.
- `include("relation")`: Eager-load relations (see Relations chapter).
- `with_deleted()`: Include soft-deleted rows.
- `all()`: Execute and return `Vec<Model>`.
- `update(json)`: Bulk update.
- `delete()`: Bulk delete or soft delete.

## Filters and Safety

Prefer the parameterized filter helpers (`filter_eq`, `filter_gt`, etc.) when
values come from users. Raw filters are still available, but they are unsafe
for untrusted input and require an explicit `.allow_unsafe()` flag.

Example with bound parameters:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let rows = User::find_in_pool(&pool)
    .filter_gte("age", 18)
    .all()
    .await?;
# Ok(())
# }
```

Raw filters (`filter`/`filter_raw`) accept SQL fragments. This keeps the builder
fast and small, but it also means **you are responsible for SQL safety** and must
explicitly opt in with `.allow_unsafe()`. Prefer:

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
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let rows = User::raw_sql("SELECT * FROM users WHERE age > 18")
    .fetch_all(&pool)
    .await?;
# Ok(())
# }
```

## Raw Struct Mapping

For reporting queries that do not map to a model, use `Premix::raw(...).fetch_as::<T>()`:

```rust,no_run
use premix_orm::prelude::*;

#[derive(sqlx::FromRow)]
struct ReportRow {
    status: String,
    count: i64,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
let rows = Premix::raw("SELECT status, COUNT(*) as count FROM users GROUP BY status")
    .fetch_as::<premix_orm::sqlx::Sqlite, ReportRow>()
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
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
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
