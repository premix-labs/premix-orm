# Security Audit

Date: 2026-01-31
Scope: premix-core/src/query.rs, premix-core/src/executor.rs, premix-macros/src

## Injection Vectors
- Raw filters (`filter`/`filter_raw`) accept unescaped SQL when `allow_unsafe` is enabled. This is intended but must be clearly documented as unsafe. (premix-core/src/query.rs)
- Column names are quoted via SqlDialect::quote_identifier for structured filters, reducing injection risk. (premix-core/src/query.rs)

## Panic Log
- No panics observed in the main query builder paths; errors return `sqlx::Error::Protocol` for unsafe usage. (premix-core/src/query.rs)

## Unsafe Flags
- `unsafe impl Send/Sync for Executor` relies on DB connection Send/Sync contracts; safety comments exist but should be periodically audited. (premix-core/src/executor.rs)

## Resource Risks
- `filter_in` accepts arbitrary list lengths; large vectors can create huge SQL and bind lists, risking memory/time blowups. (premix-core/src/query.rs)
- `raw_sql` allows arbitrary SQL; can bypass safety guards and should be used with care. (premix-core/src/model.rs)

## Fuzz Coverage
- Property-based tests cover identifier quoting and empty `filter_in` lists only; no fuzzing for raw filters or extreme bind sizes. (premix-core/tests/security_fuzz.rs)

Findings count: 5 (includes design trade-offs that should be documented).
