# MIGRATION AUDIT

Status: Partial (premix-cli tests executed for SQLite; Postgres/MySQL not exercised).

Checklist:
- Diff accuracy: indexes/constraints/defaults. (SQLite tests cover missing columns/indexes)
- Anti-drop guard: rename vs drop safety. (Not exercised in CLI tests)
- Dialect mapping: SQLite/Postgres/MySQL. (SQLite tested; Postgres/MySQL not run)
- Transactional safety: atomic/idempotent migrations. (SQLite migrator tests cover apply/rollback)

Evidence:
- `cargo test -p premix-cli` ran and passed (SQLite migration apply/rollback covered).
- `cargo test -p premix-core` ran and passed (SQLite schema diff and migrator tests covered).
- `cargo test -p premix-core --features postgres` ran and passed (Postgres schema/migrator tests covered).

Recommendation:
- Run schema/migration audits against live Postgres/MySQL before release.
