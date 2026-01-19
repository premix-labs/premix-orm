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
    let pool = premix_orm::sqlx::SqlitePool::connect(&db_url).await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Ok(())
}
```
