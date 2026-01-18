# Migrations

Premix offers two ways to manage schemas:

1. `Premix::sync` for auto-create tables from models.
2. Versioned SQL migrations via the CLI.

## Auto Sync

```rust
Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
```

This uses the model-generated `CREATE TABLE IF NOT EXISTS ...` SQL.

## CLI Migrations

Create a migration:

```bash
premix migrate create create_users
# migrations/20260118000000_create_users.sql
```

Edit the SQL file:

```sql
-- up
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);

-- down
DROP TABLE users;
```

Apply pending migrations:

```bash
premix migrate up
```

Notes:

- `premix migrate down` is not implemented yet.
- The CLI migrate command currently targets SQLite by default.
- The CLI reads `DATABASE_URL` if you do not pass `--database`.
