# Models

Models are normal Rust structs with `#[derive(Model)]`.

```rust
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}
```

## Table Naming

The default table name is the struct name in lowercase with an `s` suffix:
`User` -> `users`.

## Fields and Columns

`create_table_sql()` uses a simple heuristic based on field names:

- Fields ending with `_id` map to `INTEGER`.
- `name`, `title`, `status`, `email`, `role` map to `TEXT`.
- `age`, `version`, `price`, `balance` map to `INTEGER`.
- `is_active` maps to `BOOLEAN`.
- `deleted_at` maps to `TEXT`.
- Everything else maps to `TEXT`.

This keeps schema generation predictable, but it is not a full type mapping
system yet. The mapping is based on field names, not Rust types.

If you need precise control over column types, prefer explicit SQL migrations
for production schemas.

## Ignoring Fields

You can include non-column fields for in-memory data (such as relations)
by marking them with `#[premix(ignore)]`.

```rust
#[derive(Model)]
struct User {
    id: i32,
    name: String,

    #[premix(ignore)]
    in_memory_only: Option<String>,
}
```

## Soft Delete

If the struct contains a field named `deleted_at`, Premix treats the model
as soft-deletable. Deletes will update `deleted_at` instead of removing rows,
and default queries will filter out deleted rows. Use `.with_deleted()` to
include them.

## Optimistic Locking

If the struct contains a field named `version`, Premix uses optimistic
locking on `update()`. If the update fails, you will receive
`UpdateResult::VersionConflict` or `UpdateResult::NotFound`.
