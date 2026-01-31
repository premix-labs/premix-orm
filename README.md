![Premix ORM Banner](assets/premix-orm-banner.svg)

# Premix ORM

> **"Write Rust, Run Optimized SQL."**

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![crates.io](https://img.shields.io/crates/v/premix-orm.svg)](https://crates.io/crates/premix-orm)
[![docs.rs](https://img.shields.io/badge/docs.rs-premix--orm-blue)](https://docs.rs/premix-orm)

<!-- ![Premix ORM Logo](assets/premix-orm-logo.svg) -->

Premix is a **Zero-Overhead, Type-Safe ORM** for Rust that eliminates the need for manual migration files. It combines the ease of use of Active Record with the raw performance of handcrafted SQL.

> **Status (Alpha / Research Prototype)**  
> Premix is a research prototype. APIs may change and production use is not recommended yet.  
> This codebase is an AI-assisted research prototype and should be treated as experimental.  
> See [DISCLAIMER.md](DISCLAIMER.md).  
> Note: Versions 1.0.0-1.0.4 were published before we added the `-alpha` suffix, but they are still considered alpha.

## Why People Try Premix

- **Fast like raw `sqlx`**: generated SQL with a minimal runtime layer.
- **Low ceremony**: `save`, `find`, `include` with a simple model.
- **Transparent SQL**: inspect `to_sql()` before running anything.

## Requirements

- Rust 1.85+ (edition 2024).
- No nightly toolchain required.

## Core Philosophy (Short)

- **Zero-Overhead**: treat ORM as a thin layer on top of raw SQL.
- **Mental Model Match**: code should look like how you think about data.
- **Impossible-to-Fail**: push more errors to compile time.
- **Glass Box**: show the SQL you are about to run.
- **Escape Hatch**: allow raw SQL when needed.

Why this is practical:

- **Zero-Overhead**: benchmarks show Premix close to raw `sqlx` in common CRUD flows (see below).
- **Mental Model Match**: `user.save()` / `User::find_in_pool()` avoids boilerplate glue.
- **Impossible-to-Fail**: model fields are validated at compile time; schema mismatches surface early.
- **Glass Box**: `to_sql()`/`to_update_sql()` let you inspect SQL before running it.
- **Escape Hatch**: `Model::raw_sql()` gives full control for edge cases.

See [docs/plan/DEVELOPMENT.md](docs/plan/DEVELOPMENT.md) for the engineering flowplan and
[docs/plan/PHILOSOPHY_CHECKLIST.md](docs/plan/PHILOSOPHY_CHECKLIST.md) for status details.

## Why Premix?

- **Auto-Sync Schema:** Premix syncs your Rust structs directly to the database for rapid prototyping. No manual SQL required.
- **Zero Overhead:** Uses Rust Macros to generate SQL at compile-time. No runtime reflection.
- **Application-Level Joins:** Solves the N+1 problem using smart `WHERE IN` clauses instead of complex SQL JOINs, making scaling easier.
- **Multi-Database:** Write once, run on **SQLite**, **PostgreSQL**, or **MySQL** (feature-gated).
- **Web & Metrics:** Built-in support for **Axum**, **Actix-web**, and **Prometheus** (optional features).

---

## Benchmarks (Latest Results)

We don't just say we're fast; we prove it.

TL;DR: Premix is near raw `sqlx` for inserts/selects and dramatically faster
than loop-based bulk updates in this benchmark suite.

Highlights (Criterion medians from the latest run; see `docs/bench/BENCHMARK_RESULTS.md`):

- Insert (1 row): Premix **57.345 us** vs raw SQLx **55.022 us**
- Select (1 row): Premix **53.015 us** vs raw SQLx **53.32 us**
- Bulk Update (1,000 rows): Premix **91.608 us** vs loop **67.645 ms** (~739x faster)
- Postgres SELECT: Premix **78.659 us** vs raw SQL **71.325 us**

Full results: [docs/bench/BENCHMARK_RESULTS.md](docs/bench/BENCHMARK_RESULTS.md)

| Operation            | Premix       | SeaORM   | Rbatis   | SQLx (Raw)   |
| -------------------- | ------------ | -------- | -------- | ------------ |
| **Insert**           | 57.345 us    | 64.654 us | 29.894 us | **55.022 us** |
| **Select**           | **53.015 us** | 59.163 us | 31.64 us  | 53.32 us      |
| **Bulk Update (1k)** | **91.608 us** | -        | -        | 67.645 ms\*   |

_> Compared to standard loop-based updates._

---

## Implementation Roadmap

- [x] **Phase 1-3: Foundation** (CRUD, Relations, Migrations)
- [x] **Phase 4-7: Production Baseline** (CLI, Docs, DevOps, Multi-DB)
- [ ] **Phase 4-6 Additions** (Test utilities, scaffolding, integrations, guards, masking)
- [ ] **Phase 8: Scalability** (Read/Write splitting, metrics)
- [ ] **Phase 9: Advanced Relations** (Polymorphic) (Deferred)
- [ ] **Phase 10: Legacy Support** (Composite Keys)

---

## Migrations (New in v1.0!)

Premix now supports traditional versioned migrations for production environments.

### 1. Create a Migration

```bash
premix migrate create add_users
# Created: migrations/20260118000000_add_users.sql
```

### 2. Edit SQL

```sql
-- migration/2026xxx_add_users.sql
-- up
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);

-- down
DROP TABLE users;
```

### 3. Run Migrations

```bash
premix migrate up
# Applying migration: 20260118000000_add_users
# Migrations up to date.
```

---

## Quick Start

### 1. Define Your Model

```rust
use premix_orm::prelude::*;
// No need to import premix_core or premix_macros separately!

#[derive(Model)]
struct User {
    id: i32,
    name: String,
    age: i32,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}
```

### 2. Auto-Sync Schema

```rust
// Connect to SQLite (or Postgres!)
let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

// This line creates tables automatically.
Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;
```

### 3. Fluent Querying (No N+1)

```rust
let users = User::find_in_pool(&pool)
    .include(User::posts)   // Typed relation name to avoid typos
    .filter_gt("age", 18)    // Safe parameterized filter
    .limit(20)
    .all()
    .await?;

// Sugar for common cases:
let all_users = User::all(&pool).await?;
let by_id = User::find_by_id(&pool, 1).await?;
let updated = User::update_by_id(&pool, 1, serde_json::json!({ "name": "New" })).await?;
```

---

## 5-Minute Demo

```bash
cargo new premix-demo
cd premix-demo
```

```toml
[dependencies]
premix-orm = { version = "1.0.9-alpha", features = ["postgres", "axum"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

Use the code from Quick Start and run:

```bash
cargo run
```

If you want a copy-pasteable `src/main.rs`, use:

```rust
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    #[premix(rename = "full_name")]
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,

    #[premix(sensitive)]
    password_hash: String,
}

#[derive(Model)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;

    let users = User::find_in_pool(&pool)
        .include(User::posts)
        .filter_gt("id", 0)
        .all()
        .await?;

    println!("Loaded {} users", users.len());
    Ok(())
}
```

Prefer a template? Start from `examples/basic-app` and modify as needed.

---

## Documentation

For a longer-form guide, see [orm-book/](orm-book/) in this repository. It covers
models, queries, relations, migrations, transactions, and limitations. For a map
of the project layout, see [docs/guide/PROJECT_STRUCTURE.md](docs/guide/PROJECT_STRUCTURE.md).
For transparency details, see [orm-book/src/glass-box.md](orm-book/src/glass-box.md).

## Running Examples

Examples live under `examples/`. Start with:

```bash
cd examples/basic-app
cargo run
```

## Architecture (At a Glance)

![Premix ORM Architecture](assets/premix-orm-architecture.svg)

Release notes live in [CHANGELOG.md](CHANGELOG.md), and the development roadmap is in
[docs/plan/DEVELOPMENT.md](docs/plan/DEVELOPMENT.md).

## How Premix Differs (Flow)

![Premix vs Typical ORM Flow](assets/premix-orm-flow-compare.svg)

## What `#[derive(Model)]` Generates

- `table_name()`, `create_table_sql()`, `list_columns()`
- CRUD helpers (`save`, `find_by_id`, `update`, `delete`)
- Query builder entry points (`find_in_pool`, `find_in_tx`)
- Relation helpers (`has_many`, `belongs_to`) and eager loading (`include`)

See [orm-book/models.md](orm-book/src/models.md) and
[orm-book/queries.md](orm-book/src/queries.md) for the generated API surface and
SQL inspection helpers. For macro expansion and SQL flow, see
[orm-book/src/glass-box.md](orm-book/src/glass-box.md).

## Advanced Features

### Axum Integration

Use `PremixState` as your Axum state wrapper:

```rust
use axum::{routing::get, Router};
use premix_orm::{prelude::*, PremixState};

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let app = Router::new()
        .route("/health", get(health))
        .with_state(PremixState::new(pool));

    Ok(())
}
```

### Soft Deletes

Never accidentally lose data again.

```rust
use premix_orm::prelude::*;

#[derive(Model)] // <--- Auto-detected by field name!
struct User {
    id: i32,
    deleted_at: Option<String>,
}

// Logical delete (sets deleted_at)
user.delete(&pool).await?;

// Fetch only active users (default)
let active = User::find_in_pool(&pool).all().await?;

// Fetch everyone, including deleted
let all = User::find_in_pool(&pool).with_deleted().all().await?;
```

### Bulk Operations

Update thousands of rows in microseconds.

```rust
use premix_orm::prelude::*;
use serde_json::json;

// Set all inactive users to 'archived' status
User::find_in_pool(&pool)
    .filter_lt("last_login", "2023-01-01")
    .update(json!({ "status": "archived" }))
    .await?;
// Time: ~50us (Lightning fast!)
```

### SQL Transparency

Inspect the SQL generated by the query builder.

```rust
let query = User::find_in_pool(&pool).filter_gt("age", 18).limit(10);
println!("{}", query.to_sql());
```

### Raw SQL Escape Hatch

Run raw SQL and map results to your model.

```rust
let users = User::raw_sql("SELECT * FROM users WHERE active = 1")
    .fetch_all(&pool)
    .await?;
```

### ACID Transactions

```rust
let mut tx = pool.begin().await?;

user.balance += 100;
user.save(&mut *tx).await?; // Pass transaction reference

tx.commit().await?;
```

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
premix-orm = { version = "1.0.9-alpha", features = ["postgres", "axum"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Feature Flags

| Feature    | Description                                               |
| :--------- | :-------------------------------------------------------- |
| `sqlite`   | Enable SQLite support (default)                           |
| `postgres` | Enable PostgreSQL support                                 |
| `mysql`    | Enable MySQL support                                      |
| `axum`     | Enable Axum integration (`PremixState`)                   |
| `actix`    | Enable Actix-web integration (`PremixData`)               |
| `metrics`  | Enable Prometheus metrics (`install_prometheus_recorder`) |

## Common Pitfalls

- **Feature flags**: enable database and integration features on `premix-orm`.
- **`DATABASE_URL`**: CLI uses it by default; pass `--database` if needed.
- **`migrate down`**: requires a valid `-- down` section in your migration file.
- **Bulk update examples**: add `serde_json` if you use `json!` helpers.
- **Save vs update**: `save()` updates when `id` is set and inserts when `id == 0`.
- **Error mapping**: `ModelResultExt` maps `sqlx::Error` into `PremixError`.

## Safety Rails (Quick Notes)

- Raw filters (`filter_raw`) require `.allow_unsafe()` and log `RAW(<redacted>)`.
- Bulk update/delete require filters unless you explicitly allow unsafe paths.
- `unsafe_fast()` skips logging/metrics and safety guards by design.

## Performance Tuning

- Use `include()` for N+1 avoidance and prefer batched bulk updates when possible.
- Inspect SQL with `to_sql()`/`to_update_sql()` and keep filters parameterized.
- Tune pool sizes via `Premix::smart_*_pool_with_profile` in production.
- Use `ColumnRef::static_str("col")` for hot filters to avoid allocating column names.
- For extreme hot paths, consider `Model::raw_sql(...)` or `sqlx::query_as!` and map manually.
- Prepared statements are enabled by default; use `.unprepared()` to disable per query.

### Fast Path (Hot Queries)

For hot paths, use the fast path APIs to reduce runtime overhead:

```rust
use serde_json::json;

let users = User::find_in_pool(&pool)
    .filter_eq("status", "active")
    .fast() // skip query logging + metrics
    .all()
    .await?;

User::find_in_pool(&pool)
    .filter_eq("status", "inactive")
    .unsafe_fast() // skip logging + metrics + safety guards
    .update(json!({ "status": "active" }))
    .await?;

// Ultra-fast path (skips logging/metrics/guards AND eager loading)
let users = User::find_in_pool(&pool)
    .filter_eq("status", "active")
    .ultra_fast()
    .all()
    .await?;

// Ultra-fast writes (skips hooks/extra checks; may skip RETURNING on Postgres)
let mut user = User { id: 1, name: "Alice".to_string() };
user.save_ultra(&pool).await?;
```

### Static Query Path (Critical Operations)

Use `premix_query!` for critical paths to avoid runtime SQL generation:

```rust
// Find by id (SELECT + LIMIT 1)
let user = premix_query!(User, FIND, filter_eq("id", user_id))
    .fetch_optional(&pool)
    .await?;

// Update by id
premix_query!(
    User,
    UPDATE,
    set("status", "active"),
    filter_eq("id", user_id)
)
.execute(&pool)
.await?;

// Update by id and return the updated row
let user = premix_query!(
    User,
    UPDATE,
    set("status", "active"),
    filter_eq("id", user_id),
    returning_all()
)
.fetch_one(&pool)
.await?;

// Delete by id
premix_query!(
    User,
    DELETE,
    filter_eq("id", user_id)
)
.execute(&pool)
.await?;
```

`premix_query!` bypasses runtime SQL building and is the recommended path for
latency-sensitive code.

### Mapping Hot Path (Positional Decode)

If you need even lower mapping overhead, use `raw_sql_fast` and select columns
in model field order:

```rust
use premix_orm::prelude::*;

let rows = User::raw_sql_fast("SELECT id, name FROM users ORDER BY id")
    .fetch_all(&pool)
    .await?;
let users: Vec<User> = rows.into_iter().map(|row| row.into_inner()).collect();
```

## Deployment and CI

- Use versioned SQL migrations for production environments.
- In CI/Docker, run `premix migrate up` before application start.
- See [orm-book/migrations.md](orm-book/src/migrations.md) and
  [orm-book/production-checklist.md](orm-book/src/production-checklist.md).

## Comparisons (High-Level)

- **Premix vs Diesel:** Premix favors runtime simplicity and transparent SQL; Diesel offers
  a richer compile-time query DSL with more compile-time overhead.
- **Premix vs SeaORM:** Premix trades some dynamic query flexibility for a thinner runtime
  layer and simpler mental model.
- **Premix vs raw SQLx:** Premix adds schema/relations conveniences while keeping SQL visible.

## Compatibility

- Uses the Tokio runtime (async/await).
- `sqlx` features must match your target database.

## Limitations

See [orm-book/limitations.md](orm-book/src/limitations.md) for current gaps
and known constraints.

## Performance Tuning (More)

See [orm-book/src/performance-tuning.md](orm-book/src/performance-tuning.md) for
advanced tuning tips and hot-path options.

## Example App

Try `examples/basic-app` for a minimal runnable setup.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
