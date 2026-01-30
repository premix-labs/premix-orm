# Glass Box: How Premix Generates SQL

Premix aims to be transparent. This page shows how SQL is generated and where
it flows at runtime.

## The Pipeline (High Level)

1) Rust struct + `#[derive(Model)]`
2) Generated trait impls and helpers
3) QueryBuilder or `premix_query!`
4) `sqlx::query` / `sqlx::query_as`
5) Driver execution

There is no runtime reflection layer. SQL strings are built either at compile
 time (`premix_query!`) or with minimal runtime formatting.

## Inspecting SQL at Runtime

QueryBuilder exposes SQL inspection helpers:

```rust
let query = User::find_in_pool(&pool)
    .filter_eq("status", "active")
    .limit(10);

println!("{}", query.to_sql());
```

Update and delete paths also expose SQL builders:

```rust
println!("{}", query.to_update_sql());
println!("{}", query.to_delete_sql());
```

## Compile-Time SQL with `premix_query!`

`premix_query!` builds SQL at compile time and returns a `sqlx::Query` or
`sqlx::QueryAs`:

```rust
let user = premix_query!(User, FIND, filter_eq("id", user_id))
    .fetch_optional(&pool)
    .await?;

premix_query!(
    User,
    UPDATE,
    set("status", "active"),
    filter_eq("id", user_id)
)
.execute(&pool)
.await?;
```

## Macro Expansion (Optional)

If you want to inspect the generated Rust, use:

```bash
cargo expand -p premix-core
```

This is useful for validating the generated SQL and ensuring no unexpected
allocations are introduced in hot paths.

## Prepared Statement Caching

Premix uses `sqlx` prepared statement caching by default. You can disable it
per query if you need raw, non-prepared execution:

```rust
let users = User::find_in_pool(&pool)
    .filter_eq("status", "active")
    .unprepared()
    .all()
    .await?;
```

## Logging and Safety

- `fast()` skips logging/metrics.
- `unsafe_fast()` skips logging/metrics and safety guards.
- `ultra_fast()` also skips eager loading.

Use these when you fully control the inputs.
