# Limitations and Notes

This section documents current constraints so expectations are clear.

## Schema Generation

- Table names are derived as `<struct_name_lowercase>s`.
- Column types are inferred from field names, not Rust types.
- There is no custom column type mapping yet.

## Query Builder

- Only `filter`, `limit`, `offset`, `include`, `with_deleted`, `all`, `update`,
  and `delete` are supported today.
- There is no built-in `order_by` or select projection API.

## Relations

- Only `has_many` and `belongs_to` are available.
- Eager loading uses application-level batching (no SQL JOIN API).

## Migrations

- The CLI currently targets SQLite by default.
- `premix migrate down` is not implemented yet.

## Soft Delete

- Soft delete is enabled by a field named `deleted_at` and updates it with the
  database `CURRENT_TIMESTAMP` function.

These are good areas for future contributions and extensions.
