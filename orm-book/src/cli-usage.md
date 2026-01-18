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

Create a migration:

```bash
premix migrate create create_users
```

Apply migrations:

```bash
premix migrate up
```

Note: `premix migrate down` is not implemented yet.

By default, the CLI reads `DATABASE_URL` or falls back to `sqlite:premix.db`.
You can pass a database directly:

```bash
premix migrate up --database sqlite:my_app.db
```

## Sync (Experimental)

```bash
premix sync
```

The CLI sync command is a placeholder. Prefer calling
`Premix::sync::<DB, Model>(&pool)` in your application code.
