# Bulk Ops and Soft Delete

Premix provides bulk update and delete operations on the query builder. These
are designed for simple, fast batch changes with predictable SQL.

## Bulk Update

```rust,no_run
use premix_orm::prelude::*;
use serde_json::json;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
    status: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let updated = User::find_in_pool(&pool)
    .filter_gt("age", 18)
    .update(json!({ "status": "active" }))
    .await?;
# Ok(())
# }
```

`update()` builds a single SQL `UPDATE` statement and returns the number of
rows affected. Only JSON objects are accepted as the update payload.

### Safety Rules

- Calling `update()` without a filter is rejected.
- Unsupported JSON value types are rejected.

## Bulk Delete

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    status: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let removed = User::find_in_pool(&pool)
    .filter_eq("status", "banned")
    .delete()
    .await?;
# Ok(())
# }
```

If the model has `deleted_at`, this performs a soft delete by updating that
column. Otherwise, it performs a hard delete.

### Safety Rules

- Calling `delete()` without a filter is rejected.

## Soft Delete Queries

When `deleted_at` is present, default queries exclude deleted rows. Use
`.with_deleted()` to include them:

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
let all = User::find_in_pool(&pool)
    .with_deleted()
    .all()
    .await?;
# Ok(())
# }
```

## When to Use Bulk Ops

- **Batch status updates** or soft deletes.
- **Data cleanup** jobs.
- **Backfills** with simple conditions.

If you need complex conditions or JOINs, use raw SQL for full control.
