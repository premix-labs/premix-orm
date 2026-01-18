# Bulk Ops and Soft Delete

## Bulk Update

```rust
use serde_json::json;

let updated = User::find_in_pool(&pool)
    .filter("age > 18")
    .update(json!({ "status": "active" }))
    .await?;
```

`update()` builds a single SQL `UPDATE` statement and returns the number of
rows affected.

## Bulk Delete

```rust
let removed = User::find_in_pool(&pool)
    .filter("status = 'banned'")
    .delete()
    .await?;
```

If the model has `deleted_at`, this performs a soft delete by updating that
column. Otherwise, it performs a hard delete.

## Soft Delete Queries

When `deleted_at` is present, default queries exclude deleted rows. Use
`.with_deleted()` to include them:

```rust
let all = User::find_in_pool(&pool)
    .with_deleted()
    .all()
    .await?;
```
