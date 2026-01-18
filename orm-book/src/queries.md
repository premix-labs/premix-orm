# Queries

Premix uses a small query builder focused on simple, fast queries.

## Find by ID

```rust
let user = User::find_by_id(&pool, 1).await?;
```

## Query Builder

```rust
let users = User::find_in_pool(&pool)
    .filter("age > 18")
    .limit(20)
    .offset(0)
    .all()
    .await?;
```

Supported methods:

- `filter("...")` appends SQL to the `WHERE` clause.
- `limit(n)` and `offset(n)` apply pagination.
- `include("relation")` triggers eager loading for relations.
- `with_deleted()` includes soft-deleted rows.

Notes:

- `filter()` accepts raw SQL fragments. Use bound parameters in your own
  queries where possible to avoid SQL injection.
- There is no built-in `order_by` or select projection API yet.

## Bulk Update and Delete

These are covered in detail in the bulk ops chapter, but they are exposed on
the query builder as `update()` and `delete()`.
