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

This currently prints a confirmation message and does not scaffold files.

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

### Revert Last Migration

```bash
premix migrate down
```

### Database Selection

By default, the CLI reads `DATABASE_URL` or falls back to `sqlite:premix.db`.
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

The CLI sync command looks for a `src/bin/premix-sync.rs` binary in your
project and runs it. Use that binary to call `Premix::sync` for your models.

Example `src/bin/premix-sync.rs`:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = Premix::smart_sqlite_pool(&db_url).await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Ok(())
}
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
file. These commands require a `src/bin/premix-schema.rs` binary that uses
`premix_core::schema`.

```bash
premix schema diff --database sqlite:my_app.db
premix schema migrate --database sqlite:my_app.db --out migrations/20260101000000_schema.sql
premix schema diff --database postgres://localhost/my_app
premix schema migrate --database postgres://localhost/my_app --out migrations/20260101000000_schema.sql
```

Example `src/bin/premix-schema.rs`:

```rust,no_run
use premix_orm::prelude::*;
use premix_orm::schema;
use premix_orm::schema_models;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")?;
    let expected = schema_models![User];
    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        let pool = Premix::smart_postgres_pool(&db_url).await?;
        let diff = schema::diff_postgres_schema(&pool, &expected).await?;
        println!("{}", schema::format_schema_diff_summary(&diff));
        let sql = schema::postgres_migration_sql(&expected, &diff).join("\n");
        if !sql.trim().is_empty() {
            println!("{}", sql);
        }
        return Ok(());
    }

    let pool = Premix::smart_sqlite_pool(&db_url).await?;
    let diff = schema::diff_sqlite_schema(&pool, &expected).await?;
    println!("{}", schema::format_schema_diff_summary(&diff));
    let sql = schema::sqlite_migration_sql(&expected, &diff).join("\n");
    if !sql.trim().is_empty() {
        println!("{}", sql);
    }
    Ok(())
}
```
