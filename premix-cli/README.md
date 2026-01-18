# Premix CLI

The official command-line tool for **Premix ORM**.

`premix-cli` provides essential utilities for managing your database schema, creating migrations, and synchronizing your Rust models with the database.

## Installation

```bash
cargo install premix-cli
```

## Usage

### 1. Initialize a Project
Currently a placeholder for future scaffolding.

```bash
premix init
```

### 2. Manage Migrations

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

### 3. Sync Schema (Experimental)
Synchronize your Rust `#[derive(Model)]` structs with the database schema implicitly.

```bash
premix sync
```
*Note: For robustness, we recommend using `Premix::sync(&pool)` in your application code on startup.*

## Configuration

The CLI primarily relies on the `DATABASE_URL` environment variable.

```bash
# Example .env or shell export
export DATABASE_URL="sqlite:premix.db?mode=rwc"
```

## License

This project is licensed under the [MIT license](LICENSE).
