# DX & Safety Audit

Date: 2026-01-31
Scope: premix-core/src/query.rs, premix-macros/src/lib.rs, docs/

## Friction Points (top 3)
1) Stringly-typed column names (`filter_eq("name", ...)`) allow typos without compile-time checks. (premix-core/src/query.rs)
2) `unsafe_fast()` implicitly enables `allow_unsafe`, which makes it easy to bypass safety checks without noticing. (premix-core/src/query.rs)
3) Raw filters require `allow_unsafe`, but documentation does not emphasize the logging redaction and risk trade-offs in one place. (premix-core/src/query.rs, docs/)

## Safety Breach Report
- No critical breach found in default paths: bulk update/delete require filters or `allow_unsafe`, and raw filters are blocked without opt-in. (premix-core/src/query.rs)
- Risk surface: `unsafe_fast()` disables logging/metrics and safety guards by design.

## IDE Compatibility
- Derive macros provide a predictable API surface; code completion works once the macro is expanded.
- Missing `premix_orm::schema_models` re-export causes confusion for docs and IDE resolve in CLI examples. (orm-book/src/cli-usage.md)

## Refactoring Advice
- Provide typed column refs or generated `columns::*` constants to reduce stringly-typed errors.
- Make `unsafe_fast()` require an explicit `.allow_unsafe()` call rather than toggling it implicitly.
- Add a dedicated error type to surface safety violations (now available via `ModelResultExt`, but should be highlighted in docs).
- Document that `filter_in([])` becomes `1=0` to set expectations.
- Add a short "safety rails" section in README and orm-book with examples.

Definition of Done: 5 concrete findings included.
