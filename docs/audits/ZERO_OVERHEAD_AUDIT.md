# ZERO OVERHEAD AUDIT (Performance)

Status: Partial audit complete. Tools available, macro expansion executed, benches rerun, and cargo bloat executed on examples/basic-app.

Allocation Hotspots:
- premix-macros expansion uses format! to build SQL strings for relations (runtime allocation). See macro expansion output; consider const/concat! or cached SQL per model.
- Query builder still formats filter log strings; acceptable for logging but avoid on hot path.

Dispatch Analysis:
- Macro expansion uses Result and Vec without fully qualified paths in generated code. This is a hygiene and potential shadowing risk.

Cache Efficiency:
- Prepared statements are enabled via persistent(true). Placeholder cache exists. Recommend caching static SQL strings per model/operation where possible to reduce format! costs.

Code Size (cargo bloat, examples/basic-app):
- Top symbols are dominated by sqlite and sqlx internals; premix_core pool path appears (~19 KiB in .text).
- Total .text size reported: ~3.4 MiB (binary size 4.4 MiB).

Benchmark Fairness Checklist:
- DB version and pool size must be identical across ORM and raw SQL. (Bench scripts enforce DB settings; not independently verified.)
- Prepared statements policy must be aligned across targets. (Premix uses persistent(true); verify raw/other ORMs match.)
- Mapping strategy must match (raw/static/ORM). (Bench scripts include raw/manual map paths.)

Actionable Optimizations:
1) Replace format! with const/concat! or static SQL cache in macro-generated relations.
2) Fully qualify std types in generated code to avoid user shadowing.
3) Document fairness contract in benchmark docs and enforce via scripts.

Bench Evidence:
- bench_orm.ps1 and bench_io.ps1 executed; summary generated in benchmarks/results/summary.md.

Stop Conditions:
- Full fairness review across all ORM paths not completed in this pass.
