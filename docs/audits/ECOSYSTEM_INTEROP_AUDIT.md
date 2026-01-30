# ECOSYSTEM INTEROP AUDIT

Status: Partial (feature compile checks executed).

Findings:
- `cargo check -p premix-core --features postgres` succeeded.
- `cargo check -p premix-core --no-default-features --features mysql` succeeded with warnings about unused migrator items.
- `examples/tracing-app` ran and produced structured logs with query spans and SQL statements.
- Common types (uuid/chrono/json) appear supported in code; verify in docs and tests.

Gaps:
- Runtime-agnostic guarantee not verified.
- MySQL-only build emits dead_code warnings in migrator module.
 - Tracing output includes SQL statements; verify sensitive field masking in logs.

Stop Conditions:
- Feature matrix not compiled; runtime compatibility not executed.
