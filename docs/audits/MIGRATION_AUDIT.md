# Migration Audit

Date: 2026-01-31
Scope: premix-cli/src (migration logic), premix-core/src (sync/diff), docs/

## Destructive Path Analysis
- `Premix::sync` only creates tables; it does not drop or alter existing tables. Safe for prototypes but not schema evolution. (premix-core/src/lib.rs)
- Schema diff for SQLite emits ADD COLUMN statements and TODO comments for drops/renames; no automatic DROP COLUMN or DROP TABLE, which is safer by default. (premix-core/src/schema.rs)
- CLI `migrate down` rolls back only the last migration and requires a non-empty `-- down` section; SQLite down migrations can be destructive depending on user SQL. (premix-cli/src/main.rs)

## Dialect Inconsistency Report
- Diff/migration SQL exists for SQLite and Postgres, but not for MySQL. (premix-core/src/schema.rs)
- Foreign key changes for SQLite are emitted as TODO comments (requires table rebuild). (premix-core/src/schema.rs)

## Transactional Safety
- Migrator wraps apply/rollback in a transaction for SQLite and Postgres. (premix-core/src/migrator.rs)
- CLI supports dry-run previews and confirmation flags for down migrations; this reduces accidental destructive actions. (premix-cli/src/main.rs)

## Migration Resilience Score
- 7/10. Good safety defaults and transactional behavior, but missing MySQL diff/migration generation and limited automated handling of drops/renames.

## Refactoring Needs
- Add MySQL schema diff/migration support.
- Add explicit warnings in orm-book CLI usage about SQLite down migrations and table rebuilds.
- Provide a safe rename workflow or helper to reduce manual TODO steps.

Definition of Done: 3 edge cases verified.
