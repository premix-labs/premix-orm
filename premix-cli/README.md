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

- Project initialization (placeholder scaffold)
- SQL-based migrations (create, up, down)
- Experimental schema sync command
- Schema diff/migration via `premix schema` (requires `premix-schema` bin)

## Usage

### Initialize a Project

Currently a placeholder for future scaffolding.

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

```bash
# Uses DATABASE_URL env var by default, or defaults to sqlite:premix.db
premix migrate up

# Or specify database URL locally
premix migrate up --database sqlite:my_app.db
```

#### Revert Migration (Down)

Reverts the last applied migration.

```bash
premix migrate down
```

### Sync Schema (Experimental)

Synchronize your Rust `#[derive(Model)]` structs with the database schema implicitly.

```bash
premix sync
```

The CLI looks for `src/bin/premix-sync.rs` and runs it. Use that binary to
call `Premix::sync` for the models you want to create.

_Note: For robustness, we still recommend calling `Premix::sync(&pool)` in your
application code on startup._

### Schema Diff (SQLite v1)

Diff or generate migrations from local models (SQLite v1).

```bash
premix schema diff --database sqlite:my_app.db
premix schema migrate --database sqlite:my_app.db --out migrations/20260101000000_schema.sql
```

The CLI looks for `src/bin/premix-schema.rs` and runs it. That binary should
use `premix_core::schema` to compare models to the live database and print SQL
when running `migrate`.

Recommended output includes a summary using
`premix_core::schema::format_schema_diff_summary`.

## Compatibility

- SQLite is enabled by default.
- Postgres requires `--features postgres` when building.

## Configuration

The CLI primarily relies on the `DATABASE_URL` environment variable.

```bash
# Example .env or shell export
export DATABASE_URL="sqlite:premix.db?mode=rwc"
```

## License

This project is licensed under the [MIT license](LICENSE).
