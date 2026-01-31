# Limitations and Notes

> **Note:** Premix is an AI-assisted research prototype. These limitations
> reflect the current alpha state. See [DISCLAIMER](../../DISCLAIMER.md).

This section documents current constraints so expectations are clear.

## Schema Generation

- Table names are derived as `<struct_name_lowercase>s`.
- Column types are inferred from Rust field types and naming rules; custom
  column type mapping is still limited.
- Complex composite keys and custom Postgres types are not supported yet.

## Query Builder

- Basic filters (`filter_eq`, `filter_gt`, etc.) are supported with bound values.
- Raw filters (`filter`/`filter_raw`) require `.allow_unsafe()`.
- The fluent API does not yet include a full projection builder; use
  `Premix::raw(...).fetch_as::<T>()` for reporting queries and custom SELECTs.
- Compile-time SQL is available via `premix_query!`, but it only supports
  `SELECT`, `FIND`, `INSERT`, `UPDATE`, and `DELETE`.

## Relations

- Only `has_many` and `belongs_to` are available.
- Eager loading uses application-level batching (no SQL JOIN API).
- Polymorphic relations are not supported yet.

## Migrations

- The CLI targets SQLite by default.
- `premix migrate down` reverts the most recent migration only.
- Auto-sync is intended for development and prototyping, not production.
- SQLite down migrations may require table recreation and can cause data loss.

## Runtime

- Premix targets the Tokio runtime (`sqlx` runtime-tokio). `async-std` is not supported.

## Safety Guards

- Destructive guards require explicit opt-in for unsafe operations.
- Raw SQL paths may bypass some safety checks by design.

## Performance Notes

- Premix relies on `sqlx` prepared statement caching.
- For extreme hot paths, use `premix_query!` or `raw_sql_fast` to avoid runtime
  SQL building and reduce decode overhead.

These are good areas for future contributions and extensions.
