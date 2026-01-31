# Premix CLI

The official command-line tool for **Premix ORM**.

`premix-cli` provides utilities for managing migrations and basic project setup.

## Research Status

This crate is part of an AI-assisted research prototype. APIs may change and production use is not recommended yet.

## Installation

```bash
cargo install premix-cli
```

## Features

- Project initialization (no helper binaries required)
- SQL-based migrations (create, up, down)
- Schema sync by scanning `src/` for `#[derive(Model)]`
- Schema diff/migration via `premix schema` (SQLite/Postgres)

## Usage

### Initialize a Project

Initializes the project and confirms CLI defaults.

```bash
premix init
```

### Manage Migrations

Premix ORM supports SQL-based migrations. You can create, run, and revert them using the CLI.

#### Create a Migration

Creates a new `.sql` file in the `migrations/` directory with `up` and `down` steps.

```bash
premix migrate create create_users
# Output: Created migration: migrations/20240101123456_create_users.sql
```

#### Run Migrations (Up)

Applies all pending migrations to the database.
Migrations are applied in a transaction when supported.

```bash
# Uses DATABASE_URL env var by default, or defaults to sqlite:premix.db
premix migrate up

# Or specify database URL locally
premix migrate up --database sqlite:my_app.db

# Preview pending migrations
premix migrate up --dry-run
```

#### Revert Migration (Down)

Reverts the last applied migration.

```bash
premix migrate down

# Preview the migration that would be reverted
premix migrate down --dry-run

# Skip confirmation prompt
premix migrate down --yes
```

SQLite down migrations may require table recreation and can cause data loss.

### Sync Schema (Experimental)

Synchronize your Rust `#[derive(Model)]` structs with the database schema implicitly.

```bash
premix sync

# Preview without running
premix sync --dry-run
```

The CLI scans `src/` for `#[derive(Model)]` and creates missing tables for
those models. This removes the need to maintain helper binaries.

If SQLite is locked by background `rustc` processes on Windows, set
`PREMIX_SIGNAL_RUSTC=1` to let the CLI send a termination signal before retrying.

### Schema Diff (SQLite/Postgres v1)

Diff or generate migrations from local models (SQLite v1).

```bash
premix schema diff --database sqlite:my_app.db
premix schema migrate --database sqlite:my_app.db --out migrations/20260101000000_schema.sql

# Preview generated SQL without writing a file
premix schema migrate --database sqlite:my_app.db --dry-run

# Skip confirmation prompt
premix schema migrate --database sqlite:my_app.db --yes
```

The CLI scans `src/` for `#[derive(Model)]`, compares the expected schema to
the live database, and prints a summary using
`premix_core::schema::format_schema_diff_summary`.

## Compatibility

- SQLite is enabled by default.
- Postgres requires `--features postgres` when building.

## Configuration

The CLI primarily relies on the `DATABASE_URL` environment variable (auto-loaded from `.env`).

```bash
# Example .env or shell export
export DATABASE_URL="sqlite:premix.db?mode=rwc"
```

## License

This project is licensed under the [MIT license](LICENSE).
