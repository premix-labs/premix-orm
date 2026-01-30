# Performance Tuning

This page focuses on practical levers to reduce overhead in hot paths.

## Prepared Statements (Default)

Premix enables `sqlx` prepared statement caching by default. This avoids
re-parsing SQL on each call. If you need to disable prepared statements for a
specific query, use:

```rust
let users = User::find_in_pool(&pool)
    .filter_eq("status", "active")
    .unprepared()
    .all()
    .await?;
```

## Fast Path

- `fast()` skips logging/metrics.
- `unsafe_fast()` skips logging/metrics and safety guards.
- `ultra_fast()` also skips eager loading.

Use these only when you fully trust inputs and want minimal overhead.

## Static Query Path

For critical operations, prefer `premix_query!` to avoid runtime SQL building:

```rust
premix_query!(User, FIND, filter_eq("id", user_id))
    .fetch_one(&pool)
    .await?;
```

## Mapping Hot Path

Use `raw_sql_fast` to decode rows by position with minimal overhead:

```rust
let rows = User::raw_sql_fast("SELECT id, name FROM users ORDER BY id")
    .fetch_all(&pool)
    .await?;
let users: Vec<User> = rows.into_iter().map(|row| row.into_inner()).collect();
```

## Eager Loading Strategy

Premix uses an adaptive strategy for eager loading:

- Small relation sets: sorted vectors for cache locality.
- Large relation sets: hash maps for fast grouping.

You can still opt out of eager loading entirely with `ultra_fast()`.

## Pool Tuning

- Use `Premix::smart_sqlite_pool` for SQLite defaults.
- Keep pool sizes reasonable for your workload; smaller pools reduce contention
  on embedded databases, larger pools help on high concurrency services.

## Logging Overhead

For latency-sensitive endpoints, disable logs and metrics with `fast()` or
`unsafe_fast()`.
