# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.8-alpha] - 2026-01-31

### Added

- **Schema:** Added MySQL schema introspection + diff + migration SQL support.
- **Query Builder:** Added bind support for `NaiveDate`, `NaiveDateTime`, and JSON values.
- **Errors:** Added `PremixError` + `ModelResultExt` to map sqlx errors into domain-friendly results.
- **CLI:** Added `init` scaffolding for `premix-sync` + `premix-schema` and `.env` auto-load in CLI.
- **Docs:** Expanded CLI schema diff usage and added runtime limitations notes.

### Changed

- **Query Builder:** `unsafe_fast()`/`ultra_fast()` no longer auto-enable `allow_unsafe()`.
- **Performance:** Cached relation and insert SQL strings to reduce allocations.

### Fixed

- **Model Save:** `save()` now updates when `id != 0` and handles versioned updates more safely.
- **CLI Schema:** Schema helper template now supports SQLite/Postgres/MySQL with feature gating.

## [1.0.7-alpha] - 2026-01-30

### Added

- **Zero-Overhead Macro:** Added `premix_query!` for compile-time SQL generation (0% overhead).
- **CRUD Macros:** `premix_query!` supports `SELECT`, `FIND`, `INSERT`, `UPDATE`, `DELETE` with compile-time SQL.
- **Static Queries:** `premix_query!` now accepts `returning_all()` for UPDATE/DELETE when returning rows is required.
- **Fast Path:** Added `.fast()`/`.unsafe_fast()` on `QueryBuilder` to bypass logging/metrics and (optionally) safety guards for hot paths.
- **Ultra Fast Path:** Added `.ultra_fast()` on `QueryBuilder` plus `save_ultra`/`update_ultra`/`delete_ultra` to minimize overhead on critical paths.
- **Mapping Hot Path:** Added `from_row_fast` + `raw_sql_fast` for positional decoding in critical paths.
- **Prepared Statements:** Enabled per-query statement caching (`.persistent(true)`) and added `.prepared()`/`.unprepared()` on `QueryBuilder`. `smart_sqlite_pool` now sets a higher statement cache capacity.
- **Adaptive Eager Loading:** Eager relations now choose between sorted vectors and hash maps based on relation size, reducing overhead for small graphs while scaling for large ones.
- **Web Integrations:** Built-in helpers for **Axum** (`PremixState`) and **Actix-web** (`PremixData`) natively in `premix-orm`.
- **Metrics:** Integrated Prometheus metrics recorder natively into `premix-core`.
- **Benchmarks:** Updated benchmarks showing `premix_query!` matching raw `sqlx` latency exactly.
- **Benchmarks:** Added `premix_vs_sqlx` benchmark for direct comparison against raw `sqlx` latency.
- **Scripts:** Added `scripts/bench/bench_compare.ps1` and `bench_direct.ps1` for convenient benchmark runs.
- **Scripts:** Standardized all benchmark scripts (`bench_orm.ps1`, `bench_io.ps1`) to use CPU pinning and high priority for reproducible results.
- **Build:** Upgraded project to **Rust Edition 2024** with resolver v3.
- **Docs:** Added Glass Box and Performance Tuning chapters to the book.

### Fixed

- **Macros:** Resolved "expected ," parsing error in `collect_schema_specs` when handling field attributes with values (like `rename`).
- **Macros:** Improved attribute skipping logic to correctly handle `syn` 2.0 nested meta parsing.

### Changed

- **Consolidation:** Merged `premix-axum`, `premix-actix`, and `premix-metrics` into the main crates via Feature Flags to simplify maintenance.
- **Static Queries:** `premix_query!` UPDATE/DELETE now default to no `RETURNING` to avoid row decode overhead in hot paths.
- **Zero-Overhead:** Refactored `#[derive(Model)]` to use compile-time `concat!` for SQL generation, eliminating heap allocations in the hot path.
- **Performance:** Pre-allocated `String::with_capacity()` in `build_placeholders()` to avoid reallocations.
- **Performance:** Pre-allocated `Vec::with_capacity(4)` for filters and `Vec::with_capacity(2)` for includes in `QueryBuilder`.
- **Performance:** Added `#[inline]` hints on critical hot path functions (`bind_value_query_as`, `build_placeholders`).
- **Performance:** Eager loading now uses sorted `Vec` with binary search instead of `HashMap` for better L1/L2 cache locality.
- **Performance:** Wrapped debug tracing in `#[cfg(debug_assertions)]` to eliminate overhead in release builds.
- **Performance:** Cached IN-clause placeholders by `(DB, start, count)` to reduce SQL string building in hot paths.
- **Project Structure**: Major modularization of `premix-core` into `dialect`, `executor`, `model`, and `query` modules.
- **Docs:** Updated all `Cargo.toml` files with detailed Thai comments for better clarity.
- **Docs:** Added comprehensive `docs/audits/ZERO_OVERHEAD_AUDIT.md` report validating performance claims.
- **Tests:** Expanded SQLite integration suite to cover filters, fast paths, streams, SQL helpers, and guards.
- **Tests:** Added MySQL/Postgres integration coverage for filters, prepared/unprepared, raw filter guards, and streams.
- **Tests:** Added schema diff/migration tests and migrator rollback coverage for SQLite.
- **Tests:** Added metrics feature test for Prometheus recorder (feature-gated).

## [1.0.6-alpha] - 2026-01-20

> Note: Versions 1.0.0-1.0.4 were published before we added the `-alpha` suffix, but they are still considered alpha.

### Added

- **Migrations:** Implemented `migrate down` with rollback support for SQLite and Postgres.
- **CLI Sync:** `premix sync` now runs a `src/bin/premix-sync.rs` helper when present.
- **Hooks/Validation:** Added opt-in custom hooks/validation via `#[premix(custom_hooks, custom_validation)]`.
- **Docs:** Book now documents CLI sync and migrate down, plus updated hooks/validation guidance.
- **Docs:** Added a dedicated `orm-book/book-examples` compile-check crate and validation notes.
- **Benchmarks:** Added `scripts/bench/bench_repeat.ps1` to run repeatable multi-round benchmarks and save artifacts.
- **Schema:** Added SQLite schema diff engine with index/foreign key metadata and summary helpers.
- **Schema:** Added Postgres schema diff/migration support with indexes and foreign keys.
- **Schema CLI:** Added `premix schema diff/migrate` via `premix-schema` helper.
- **Query Builder:** Added parameterized filter helpers and null-check filters.
- **Relations:** Added eager loading for `belongs_to`.
- **Macros:** Added `schema_models!` and model schema metadata for indexes/foreign keys.
- **Tests:** Added coverage for schema diff, eager belongs_to, and delete_all guard paths.
- **Test Utils:** Added transactional test helper and MockDatabase.
- **Security:** Added `#[premix(sensitive)]` to redact sensitive values in query logs.
- **Safety:** Guarded `delete_all()` to require filters unless `allow_unsafe()` is set.
- **Reporting:** Added `Premix::raw(...).fetch_as::<T>()` for arbitrary struct mapping.
- **Smart Config:** Added auto-tuned pool options and smart pool helpers for SQLite/Postgres.
- **CLI:** Added `premix scaffold` to generate models from SQLite/Postgres schemas.
- **CLI:** Improved Postgres scaffolding type mapping (arrays, bytea, numeric).
- **Integrations:** Added helper crates `premix-axum` and `premix-actix`.
- **Metrics:** Added query latency hooks and pool stats recorder with Prometheus exporter.
- **Safety:** Raw SQL filters now require an explicit `.allow_unsafe()` opt-in.
- **Postgres:** `save()` now uses `RETURNING id` to sync primary keys reliably.
- **Schema:** Type inference now uses Rust field types for SQL metadata generation.
- **Performance:** Macro-generated SQL uses precomputed column lists and shared placeholder builder.

### Breaking

- **Async Traits:** Switched to `fn -> impl Future + Send` in traits for zero-overhead APIs and removed `async-trait`.
- **Eager Loading:** `Model::eager_load` now accepts `Executor` directly instead of `IntoExecutor`.

### Changed

- **CLI:** Added database selection for `premix migrate down`.
- **CLI:** Feature-gated Postgres support and updated compatibility notes.
- **Tests:** Expanded CLI and migrator tests to cover rollback flows.
- **Macros:** `#[derive(Model)]` now emits `premix-orm` paths; direct macro users must depend on `premix-orm`.
- **Benchmarks:** Standardized Criterion config (warmup/measurement/sample size) across SQLite and Postgres benches.
- **Docs:** Updated benchmark methodology/results and recorded multi-round median summaries.
- **Docs:** Updated flowplan to the Ultimate Edition and aligned README/Philosophy checklist.
- **Docs:** Updated examples and guides to prefer parameterized filters and document new schema/index/fk features.
- **Relations:** Eager loading now deduplicates IDs, chunks IN queries, and logs missing relations via tracing.

## [1.0.5-alpha] - 2026-01-19

> Note: Versions 1.0.0-1.0.4 were published before we added the `-alpha` suffix, but they are still considered alpha.

### Added

- **Migrations:** Implemented `migrate down` with rollback support for SQLite and Postgres.
- **CLI Sync:** `premix sync` now runs a `src/bin/premix-sync.rs` helper when present.
- **Hooks/Validation:** Added opt-in custom hooks/validation via `#[premix(custom_hooks, custom_validation)]`.
- **Docs:** Book now documents CLI sync and migrate down, plus updated hooks/validation guidance.
- **Docs:** Added a dedicated `orm-book/book-examples` compile-check crate and validation notes.
- **Benchmarks:** Added `scripts/bench/bench_repeat.ps1` to run repeatable multi-round benchmarks and save artifacts.
- **Safety:** Guarded `delete_all()` to require filters unless `allow_unsafe()` is set.

### Changed

- **CLI:** Added database selection for `premix migrate down`.
- **CLI:** Feature-gated Postgres support and updated compatibility notes.
- **Tests:** Expanded CLI and migrator tests to cover rollback flows.
- **Macros:** `#[derive(Model)]` now emits `premix-orm` paths; direct macro users must depend on `premix-orm`.
- **Benchmarks:** Standardized Criterion config (warmup/measurement/sample size) across SQLite and Postgres benches.
- **Docs:** Updated benchmark methodology/results and recorded multi-round median summaries.
- **Docs:** Updated flowplan to the Ultimate Edition and aligned README/Philosophy checklist.

## [1.0.4] - 2026-01-18

### Added

- **Docs:** Added research status disclaimer across README files, the book introduction, and developer docs. Added `DISCLAIMER.md`.
- **Docs:** Expanded README messaging (core philosophy summary, benchmarks highlights, badges/links, demo, pitfalls, compatibility).
- **Docs:** Added mermaid diagrams to the book for migrations and relations flows.
- **Assets:** Added product-first logo, banner, and architecture diagrams under `assets/`.
- **Docs:** Added Thai developer guide (`docs/plan/DEVELOPMENT_TH.md`).
- **Tests:** Expanded unit/integration coverage for core and CLI paths.

### Changed

- **CI:** Coverage script now excludes the proc-macro entry file for stable reporting.
- **Versioning:** Bumped crate versions and docs to 1.0.4.
- **Docs:** Replaced Mermaid diagrams with SVG assets in the book and README.
- **Docs:** Refined `docs/plan/DEVELOPMENT.md` wording for the latest flowplan narrative.

### Fixed

- **Build:** Removed duplicated `#[cfg]` attributes in `premix-core/src/lib.rs` to satisfy clippy.

## [1.0.3] - 2026-01-18

### Added

- **SQL Transparency:** Added `to_sql()`, `to_update_sql()`, and `to_delete_sql()` on the query builder for inspecting generated SQL.
- **Raw SQL Escape Hatch:** Added `Model::raw_sql(...)` for mapping raw queries to models.
- **Docs:** Added `docs/plan/PHILOSOPHY_CHECKLIST.md` and updated Core Philosophy status in `docs/plan/DEVELOPMENT.md`.

## [1.0.2] - 2026-01-18

### Fixed

- **Facade Sync:** Fixed a critical issue where the `premix-orm` facade could not find the `prelude` module in `premix-core` during publication.
- **Doctests:** Fixed all documentation examples to ensure they compile and run correctly, supporting the `premix-orm` unified entry point.

## [1.0.1] - 2026-01-18

### Added

- **Facade Crate (`premix-orm`):** Introduced a unified entry point crate. Users can now depend solely on `premix-orm` instead of managing `premix-core` and `premix-macros` separately.
- **Documentation:** Added `README.md` for `premix-core`, `premix-macros`, and `premix-cli` to ensure proper display on crates.io.
- **Benchmarks:** Expanded suite to cover **Soft Deletes** and improved fail-fast logic in `bench_orm.ps1`.

### Fixed

- **Auto-Increment IDs:** Fixed a critical bug in `save()` where explicitly setting ID to 0 prevented the database from auto-generating IDs.
- **Manual ID Support:** Improved `save()` to support manual IDs while still defaulting to auto-increment when ID is 0.
- **ID Synchronization:** Fixed a bug where the struct's `id` field wasn't updated from the database after a `save()` call.
- **Executor API:** Resolved type inference ambiguities in `IntoExecutor` for multi-database contexts.

## [1.0.0] - 2026-01-18

### Added

- **Migration System (Phase 7):**
  - `premix-cli migrate create <name>` command to generate timestamped SQL files (`YYYYMMDDHHMMSS_name.sql`).
  - `premix-cli migrate up` command to apply pending migrations.
  - `Migrator` core logic to track applied versions in `_premix_migrations` table.
- **Developer Automation:**
  - Comprehensive PowerShell script suite in `scripts/` (`check_all`, `run_fmt`, etc.).
  - Automated `organize.ps1` for managing project structure.
- **Multi-Database Support:**
  - Generic `Model<DB>` trait supporting `sqlx::Sqlite`, `sqlx::Postgres`, and `sqlx::Mysql`.
  - `SqlDialect` trait for database-specific query generation.
- **Core ORM Features:**
  - `#[derive(Model)]` macro for automatic schema mapping.
  - Active Record pattern: `save()`, `find_by_id()`, `delete()`, `update()`.
  - Fluent Query Builder: `filter()`, `limit()`, `offset()`, `order_by()`.
- **Relations:**
  - `#[has_many]` and `#[belongs_to]` macros.
  - Lazy loading (`user.posts(&pool)`).
  - Eager loading with `.include("posts")` (Batch loading strategy).
- **Advanced Features:**
  - **Soft Deletes:** `#[derive(SoftDelete)]` and `with_deleted()` API.
  - **Optimistic Locking:** Automatic `version` field handling.
  - **Bulk Operations:** `update_all(json)` and `delete_all()`.
  - **Transactions:** `save_with(&mut tx)` and `find_with(&mut tx)`.
  - **Lifecycle Hooks:** `before_save` and `after_save`.
  - JSON/JSONB support via `serde_json`.

### Changed

- Internal architecture refactored to separate `premix-core` (runtime) and `premix-macros` (compile-time).
- Improved error handling with spanned macro errors.

### Fixed

- Solved N+1 query problem using application-level joins (WHERE IN).

## [0.1.0-alpha] - 2026-01-01

- Initial Proof of Concept.
