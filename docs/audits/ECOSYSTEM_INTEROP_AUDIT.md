# Ecosystem Interop Audit

Date: 2026-01-31
Scope: premix-core/src/model.rs, premix-core/src/query.rs, docs/

## Friction Points (manual conversions)
- `BindValue` supports String/i64/f64/bool/uuid/DateTime<Utc>, but not `NaiveDateTime`, `NaiveDate`, `serde_json::Value`, or arrays; users must convert types manually. (premix-core/src/query.rs)
- Schema type mapping uses heuristics and may not align with custom database types; manual migrations required for advanced types. (orm-book/src/models.md)
- `premix_orm::schema_models` macro is referenced in docs but not re-exported, creating interop friction for CLI examples. (orm-book/src/cli-usage.md)

## Logging Gaps
- Query logging uses `tracing::debug` in save/update/select; fast/unsafe paths skip logs by design. This is acceptable but should be highlighted for observability. (premix-core/src/query.rs, premix-macros/src/lib.rs)
- Sensitive filter values are redacted, but update payloads are not explicitly redacted in logs. (premix-core/src/query.rs)

## Serde Conflicts
- No direct conflicts observed; `#[premix(ignore)]` can coexist with `#[serde(skip)]`, but guidance is not documented. (premix-core/src/model.rs)

Runtime Compatibility
- Tied to Tokio via sqlx runtime features; no async-std support documented. (premix-core/Cargo.toml, docs)

Definition of Done: 3 interop scenarios validated.
