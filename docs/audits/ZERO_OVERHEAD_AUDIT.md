# Zero-Overhead Audit

Date: 2026-01-31
Tools: cargo expand (ok), cargo bloat (failed on lib crate), bench_orm.ps1 (timeout), bench_io.ps1 (timeout), summarize_results.ps1 (ok)

## Macro Hygiene Review (premix-macros)
Expanded via `cargo expand -p premix-macros`.

### Hygiene Violations
- None found. Generated code uses fully-qualified `premix_orm::` paths and avoids name collisions.

### Allocation Hotspots
- INSERT path builds SQL with `format!` on each save call when id=0 or id!=0; no SQL string caching for insert. (premix-macros/src/lib.rs: save/save_fast)
- Relation helpers build table and FK strings with `format!` and create SQL strings at runtime per call. (premix-macros/src/relations.rs expansion)
- Query builder uses `String` to build filters and `SmallVec` growth for binds; expected but still runtime allocations in hot paths. (premix-core/src/query.rs)

### Concurrency Risks
- Generated async fns are `Send`-bounded; no non-Send captures observed in expansion. No concrete concurrency risk found.

### Refactoring Roadmap
- Cache INSERT SQL strings (separate cached forms for id=0 vs id!=0) to remove `format!` per call.
- Precompute relation table names / FK strings as `const &str` or static `OnceLock<String>` instead of `format!` per call.
- Consider a lightweight prepared SQL cache keyed by (table, column list) for save/update paths.

## Performance Audit

### Allocation Hotspots (file:line, impact, fix)
- premix-macros/src/lib.rs: save/save_fast builds SQL with `format!` each call. Impact: per-call heap alloc. Fix: cache SQL per column list.
- premix-macros/src/relations.rs: relation helper SQL strings built per call. Impact: per-call heap alloc. Fix: static SQL or cached format strings.
- premix-core/src/query.rs: filter string rendering builds `String` for logs and SQL; unavoidable but could reuse buffer for repeated calls.

### Dispatch Analysis
- Executor uses enum dispatch (match on Pool/Conn) per call; no dynamic dispatch found. (premix-core/src/executor.rs)

### Cache Efficiency Score
- 7/10. Placeholders are cached; UPDATE SQL uses OnceLock; prepared statements are reused. INSERT SQL and relation SQL are not cached.

### Instruction Overhead Estimate
- Low to moderate: per-call format/build for INSERT and relation queries adds overhead versus raw SQLx.

### Benchmark Fairness Checklist
- Same DB version/settings: Not verified in this run.
- Same pool size: Not verified in this run.
- Same prepared on/off policy: Not verified in this run.
- Same mapping strategy: Not verified in this run.
- Same query shape/dataset: Not verified in this run.

Bench Status:
- cargo bloat: failed (`only 'bin', 'dylib' and 'cdylib' crate types are supported`).
- scripts/bench/bench_orm.ps1: timed out after 120s.
- scripts/bench/bench_io.ps1: timed out after 120s.
- scripts/bench/summarize_results.ps1: succeeded and wrote benchmarks/results/summary.*.

Actionable Optimizations (min 3)
1) Cache INSERT SQL strings in save/save_fast to remove per-call `format!`.
2) Precompute relation SQL strings or cache them per relation to avoid per-call allocations.
3) Add a lightweight SQL cache for QueryBuilder `to_sql()` hot paths.
