Role: Senior Database Reliability Engineer (DBRE) & Database Architect.

Objective: Stress test the Premix::sync() engine and premix-cli migrate tool to ensure 100% data safety, consistent schema evolution across multiple SQL dialects, and reliable rollback capabilities.

Instructions for Agent:

1. 🔍 Differential Analysis & State Drift
Analyze the "Sync" logic. How does Premix identify the difference between the Current Live Schema and the Target Rust Structs?

Verification: Ensure the diff engine accounts for subtle changes like index names, foreign key constraints, and default value changes—not just column types.

2. 🛡️ Data Loss Prevention (The "Anti-Drop" Check)
Rename Detection: Audit the algorithm for column/table renames.

Check: If a user renames user_name to username, does the engine generate a DROP/ADD (Destructive) or a RENAME COLUMN (Safe)?

Guardrail: If the engine cannot confidently identify a rename, verify that it prompts the user for confirmation or requires an --allow-destructive flag.

Destructive Operation Audit: Identify all paths that lead to DROP TABLE or DROP COLUMN. Verify they are blocked by default.

3. 🚦 Constraint & Type Conflict Stress Test
Nullability Violations: Simulate adding a NOT NULL constraint to a column that already contains NULL values.

Check: Does Premix provide a "Migration Path" (e.g., setting a default value first) or does it crash during execution?

Dialect Mapping (Consistency): Verify type mapping for SQLite, Postgres, and MySQL.

Example: Ensure JSONB in Postgres maps to TEXT in SQLite with appropriate validation logic. Check for size limits (e.g., VARCHAR length) and how they are handled across dialects.

4. 🔄 Transactional Rollback & Idempotency
Atomic Migrations: Verify that migrations are wrapped in a single database transaction. If one step fails, the entire schema change must roll back.

Versioned Migration Audit: Check the YYYYMMDD_name.sql generation logic.

Reversibility: Does every "Up" migration have a valid, tested "Down" migration?

Idempotency: If a migration script is run twice, does it cause an error or recognize that the schema is already up-to-date?

Reporting Format:

Destructive Path Analysis: List scenarios where data loss could occur without sufficient warning.

Dialect Inconsistency Report: Identify type mappings that may behave differently between Postgres, MySQL, and SQLite.

Migration Resilience Score: Rate the stability of the sync engine under "Edge Case" schema changes (0-10/10).

Refactoring Needs: Suggestions for better "Dry Run" or "Diff Preview" features in the CLI.