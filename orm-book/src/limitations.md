# Limitations and Notes

This section documents current constraints so expectations are clear.

## Schema Generation

- Table names are derived as `<struct_name_lowercase>s`.
- Column types are inferred from field names, not Rust types.
- There is no custom column type mapping yet.

## Query Builder

- Basic filters (`filter_eq`, `filter_gt`, etc.) are supported with bound values.
- Raw filters (`filter`/`filter_raw`) are supported but require `.allow_unsafe()`.
- Only `limit`, `offset`, `include`, `with_deleted`, `all`, `update`, and `delete`
  are supported today.
- There is no built-in `order_by` or select projection API.
- Use `Premix::raw(...).fetch_as::<T>()` for projections and reporting queries.

## Relations

- Only `has_many` and `belongs_to` are available.
- Eager loading uses application-level batching (no SQL JOIN API).
- Polymorphic relations are not supported yet.

## Migrations

- The CLI currently targets SQLite by default.
- `premix migrate down` only reverts the most recent migration.
- Auto-sync is intended for development, not production.

## Soft Delete

- Soft delete is enabled by a field named `deleted_at`.
- Deleted rows are excluded by default; use `with_deleted()` to include them.

These are good areas for future contributions and extensions.
