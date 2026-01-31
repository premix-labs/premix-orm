# CLI Usage

The `premix` CLI helps manage migrations and basic project actions.

## Install

```bash
cargo install premix-cli
```

## Init

```bash
premix init
```

This initializes the project and confirms CLI defaults. The CLI now scans
`src/` for `#[derive(Model)]` automatically, so helper binaries are no longer
required.

## Migrations

### Create a Migration

```bash
premix migrate create create_users
```

This creates a timestamped file:

```text
migrations/20260118000000_create_users.sql
```

### Apply Migrations

```bash
premix migrate up
```

Preview pending migrations:

```bash
premix migrate up --dry-run
```

### Revert Last Migration

```bash
premix migrate down
```

Preview the migration that would be reverted:

```bash
premix migrate down --dry-run
```

Skip confirmation prompt:

```bash
premix migrate down --yes
```

> Warning: SQLite down migrations may require table recreation and can cause data loss.

### Database Selection

By default, the CLI reads `DATABASE_URL` (auto-loaded from `.env`) or falls back to `sqlite:premix.db`.
You can pass a database directly:

```bash
premix migrate up --database sqlite:my_app.db
premix migrate down --database sqlite:my_app.db
```

### Notes

- The CLI migrate command currently targets SQLite by default.
- For Postgres, build with `--features postgres` and pass a Postgres URL.

## Sync (Experimental)

```bash
premix sync
```

The CLI scans `src/` for `#[derive(Model)]` and creates missing tables for
those models.

Preview without running:

```bash
premix sync --dry-run
```

## Scaffold (Experimental)

Generate Rust models from an existing database:

```bash
premix scaffold --database sqlite:my_app.db
premix scaffold --database postgres://localhost/app --table users --out src/models.rs
```

The output includes `#[derive(Model)]` structs with basic column type mapping.
Review and refine types or add relation fields as needed.

## Schema Diff (SQLite/Postgres v1)

You can diff the database against your local models and generate a migration
file. The CLI scans `src/` for `#[derive(Model)]` and uses `premix_orm::schema`
internally (no helper binaries required). Build with `--features postgres`
when targeting Postgres.

```bash
premix schema diff --database sqlite:my_app.db
premix schema migrate --database sqlite:my_app.db --out migrations/20260101000000_schema.sql
premix schema diff --database postgres://localhost/my_app
premix schema migrate --database postgres://localhost/my_app --out migrations/20260101000000_schema.sql
```

Preview generated SQL without writing a file:

```bash
premix schema migrate --database sqlite:my_app.db --dry-run
```

Skip confirmation prompt:

```bash
premix schema migrate --database sqlite:my_app.db --yes
```

