<!-- audit-performance.md -->

Role: Senior Rust Performance Engineer & Systems Architect.

Objective: Validate the "Zero-Overhead" claim with reproducible evidence and fair comparisons versus raw sqlx.

Scope:
- Query builder hot path
- Macro-expanded code
- Eager loading data paths
- Prepared statement behavior

Prerequisites / Tools:
- cargo expand
- cargo bloat
- (optional) perf or sampling profiler

Input Matrix (must check all):
- premix-core/src/query.rs
- premix-core/src/relations.rs
- premix-macros/src/lib.rs
- premix-macros/src/relations.rs
- benchmarks/benches/premix_vs_sqlx.rs
- benchmarks/benches/orm_comparison.rs

Expected Artifacts:
- docs/audits/ZERO_OVERHEAD_AUDIT.md
- docs/bench/BENCHMARK_RESULTS.md
- benchmarks/results/summary.md

Run Order (minimal commands):
1) cargo expand -p premix-macros
2) cargo bloat -p premix-orm
3) scripts/bench/bench_orm.ps1
4) scripts/bench/bench_io.ps1
5) scripts/bench/summarize_results.ps1

Bench Fairness Contract:
- Same DB version and settings
- Same pool size
- Same prepared on/off policy
- Same mapping strategy (raw/static/ORM) per scenario
- Same query shape and dataset

Procedure:
1) Tooling check: verify cargo-expand and cargo-bloat installed.
2) Hot-path allocation scan: String/Vec growth in loops, Box/dyn/Arc in hot path.
3) Macro expansion audit: compare generated code to ideal sqlx usage.
4) Cache behavior: SQL cached per model/operation; prepared statements reused.
5) Eager loading locality: data structure adapts between Vec/HashMap by size.
6) Fairness verification: ensure contract is met.

Severity Rubric:
- Critical: correctness issue or unfair benchmark invalidates claims.
- High: measurable performance regression or major allocation in hot path.
- Medium: avoidable overhead or code bloat.
- Low: minor micro-optimizations or doc gaps.

Reporting Format:
- Allocation Hotspots (file:line, impact, fix)
- Dispatch Analysis (static vs dynamic)
- Cache Efficiency Score (brief justification)
- Instruction Overhead Estimate (qualitative)
- Benchmark Fairness Checklist (pass/fail items)

Definition of Done:
- At least 5 concrete findings with file references.
- At least 3 actionable optimizations.
- Fairness checklist completed or explicitly blocked.

Stop Conditions:
- Missing tools, missing benches, or missing outputs. Report blockers and stop.
