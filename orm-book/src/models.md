# Models

Models are normal Rust structs with `#[derive(Model)]`. The derive generates:

- Schema metadata (table name and column list).
- Basic CRUD helpers (`save`, `find_by_id`, query builder).
- Soft delete and optimistic locking behavior when fields are present.

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}
```

## Table Naming

The default table name is the struct name in lowercase with an `s` suffix:

```text
User -> users
Post -> posts
```

Use explicit SQL migrations if you need custom table names.

## Fields and Columns

`create_table_sql()` uses a name-based heuristic:

- Fields ending with `_id` map to `INTEGER`.
- `name`, `title`, `status`, `email`, `role` map to `TEXT`.
- `age`, `version`, `price`, `balance` map to `INTEGER`.
- `is_active` maps to `BOOLEAN`.
- `deleted_at` maps to `TEXT`.
- Everything else maps to `TEXT`.

This is intentionally simple and predictable. The mapping is based on field
names, not Rust types. For production schemas, prefer explicit migrations for
full control.

## ID Behavior

If your model has an `id` field, Premix treats it as the primary key. When
`id` is set to `0`, inserts use auto-increment behavior (where the database
supports it). After `save()`, the struct `id` is updated with the generated
value.

## Ignoring Fields

You can keep in-memory fields on the struct with `#[premix(ignore)]`:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,

    #[premix(ignore)]
    in_memory_only: Option<String>,
}
```

Ignored fields are not included in schema generation or SQL statements.

## Soft Delete

If the model contains a `deleted_at` field, Premix enables soft delete:

- `delete()` updates `deleted_at` instead of removing rows.
- Default queries filter out deleted rows.
- `with_deleted()` includes soft-deleted rows.

## Optimistic Locking

If the model contains a `version` field, Premix uses optimistic locking on
`update()`. If the update fails, you will receive `UpdateResult::VersionConflict`
or `UpdateResult::NotFound`.

## Relations as Fields

Use `#[premix(ignore)]` for relation fields and `#[has_many]` or
`#[belongs_to]` to generate relation helpers. See the Relations chapter for
details.

## Recommendations

- Keep models small and focused.
- Use explicit migrations for production schema evolution.
- Reserve name-based mapping for prototypes and tests.
