# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.7-alpha] - 2026-01-21

### Added

- **Web Integrations:** Built-in helpers for **Axum** (`PremixState`) and **Actix-web** (`PremixData`) natively in `premix-orm`.
- **Metrics:** Integrated Prometheus metrics recorder natively into `premix-core`.

### Changed

- **Consolidation:** Merged `premix-axum`, `premix-actix`, and `premix-metrics` into the main crates via Feature Flags to simplify maintenance.
- **Zero-Overhead:** Refactored `#[derive(Model)]` to use compile-time `concat!` for SQL generation, eliminating heap allocations in the hot path.
- **Performance:** Pre-allocated `String::with_capacity()` in `build_placeholders()` to avoid reallocations.
- **Performance:** Pre-allocated `Vec::with_capacity(4)` for filters and `Vec::with_capacity(2)` for includes in `QueryBuilder`.
- **Performance:** Added `#[inline]` hints on critical hot path functions (`bind_value_query_as`, `build_placeholders`).
- **Performance:** Eager loading now uses sorted `Vec` with binary search instead of `HashMap` for better L1/L2 cache locality.
- **Performance:** Wrapped debug tracing in `#[cfg(debug_assertions)]` to eliminate overhead in release builds.
- **Project Structure**: Major modularization of `premix-core` into `dialect`, `executor`, `model`, and `query` modules.
- **Docs:** Updated all `Cargo.toml` files with detailed Thai comments for better clarity.
- **Docs:** Added comprehensive `ZERO_OVERHEAD_AUDIT.md` report validating performance claims.

## [1.0.6-alpha] - 2026-01-20

> Note: Versions 1.0.0–1.0.4 were published before we added the `-alpha` suffix, but they are still considered alpha.

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

> Note: Versions 1.0.0ƒ?"1.0.4 were published before we added the `-alpha` suffix, but they are still considered alpha.

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
- **Docs:** Added Thai developer guide (`docs/DEVELOPMENT_TH.md`).
- **Tests:** Expanded unit/integration coverage for core and CLI paths.

### Changed

- **CI:** Coverage script now excludes the proc-macro entry file for stable reporting.
- **Versioning:** Bumped crate versions and docs to 1.0.4.
- **Docs:** Replaced Mermaid diagrams with SVG assets in the book and README.
- **Docs:** Refined `docs/DEVELOPMENT.md` wording for the latest flowplan narrative.

### Fixed

- **Build:** Removed duplicated `#[cfg]` attributes in `premix-core/src/lib.rs` to satisfy clippy.

## [1.0.3] - 2026-01-18

### Added

- **SQL Transparency:** Added `to_sql()`, `to_update_sql()`, and `to_delete_sql()` on the query builder for inspecting generated SQL.
- **Raw SQL Escape Hatch:** Added `Model::raw_sql(...)` for mapping raw queries to models.
- **Docs:** Added `docs/PHILOSOPHY_CHECKLIST.md` and updated Core Philosophy status in `docs/DEVELOPMENT.md`.

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
