# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added 🚀
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

### Changed 🔧
- Internal architecture refactored to separate `premix-core` (runtime) and `premix-macros` (compile-time).
- Improved error handling with spanned macro errors.

### Fixed 🐛
- Solved N+1 query problem using application-level joins (WHERE IN).

## [0.1.0-alpha] - 2026-01-01
- Initial Proof of Concept.
