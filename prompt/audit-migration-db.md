<!-- audit-migration-db.md -->

Role: Senior Database Reliability Engineer & Database Architect.

Objective: Validate schema sync/migration safety across SQLite/Postgres/MySQL.

Scope:
- premix::sync() diff engine
- premix-cli migrations
- destructive operations guardrails

Input Matrix (must check all):
- premix-cli/src (migration logic)
- premix-core/src (sync logic)
- docs/ (migration docs)

Expected Artifacts:
- docs/audits/MIGRATION_AUDIT.md

Procedure:
1) Diff accuracy: indexes, constraints, default changes.
2) Anti-drop guard: rename vs drop safety and allow flags.
3) Dialect consistency: mappings across SQLite/Postgres/MySQL.
4) Transactional safety: atomic and idempotent migrations.

Severity Rubric:
- Critical: data loss without explicit allow.
- High: incorrect diff or inconsistent mapping.
- Medium: missing edge-case handling.
- Low: doc gaps.

Reporting Format:
- Destructive Path Analysis
- Dialect Inconsistency Report
- Migration Resilience Score (0-10)
- Refactoring Needs (dry run, diff preview)

Definition of Done:
- At least 3 edge cases verified.

Stop Conditions:
- Missing migration tooling; report and stop.
