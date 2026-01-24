# Premix ORM: Zero-Overhead Audit Report

This report summarizes a static performance audit of Premix ORM. It focuses on
allocation behavior, dispatch patterns, eager-loading data layout, and binary
size impact versus raw sqlx usage.

Date: 2026-01-21 (local run)

---

## Executive Summary

- Premix is close to raw sqlx for CRUD, but it is not strictly zero-overhead.
- The hot path allocates small Strings and Vecs per query.
- Eager loading uses a sorted Vec + binary_search (cache-friendly), but insert
  costs rise for large relation sets.
- No dynamic dispatch in the normal query path; only stream uses BoxStream.
- Binary size is dominated by sqlx/sqlite, not Premix.

Verdict: Near-zero overhead for common CRUD, but measurable overhead remains
(allocations + a few branches). Use raw sqlx or static queries for hot paths.

---

## Tooling and Instrumentation

- cargo-expand: available
- cargo-bloat: installed during this audit

Commands used:
- cargo bloat -p basic-app -n 20 --release
- cargo bloat -p relation-app -n 20
- cargo expand -p basic-app
- cargo expand -p premix-orm --test belongs_to_eager
- scripts/bench/bench_compare.ps1 (warmup 5s, measurement 10s, CPU core pinned)

---

## 1) Allocation Hotspots (Query Path)

All line references are in premix-core/src/query.rs unless noted.

- filter_* allocates per filter:
  - filter_eq: line 236 (column/op -> String)
  - filter_gt/gte/lt/lte/like/in behave similarly
- format_filters_for_log allocates Vec<String> + join:
  - line 338
- to_sql/to_update_sql/to_delete_sql allocate String and Vec:
  - lines 428, 450, 479
- all/update/delete build SQL and where_binds Vec each call:
  - lines 618, 764, 849

Macro-generated SQL also uses runtime String building:
- premix-macros/src/lib.rs: 214-290 (set_clause + format!)

Impact: small but non-zero allocation pressure for each query.

---

## 2) Dispatch Analysis

- Query execution uses enum dispatch (Executor::Pool vs Executor::Conn):
  static and branch-predictable.
- Stream path uses BoxStream (dynamic dispatch + heap alloc):
  - premix-core/src/query.rs: 676, 708
- Tracing uses &dyn tracing::field::Value (logging path only).

No vtables in the core query builder path, but stream is not zero-overhead.

---

## 3) Cache Efficiency (Eager Loading)

Eager loading (macro-generated) uses sorted Vec + binary_search:
- premix-macros/src/relations.rs: 107-188
- Verified via cargo expand (belongs_to_eager):
  - grouped Vec + CHUNK_SIZE + binary_search_by_key

Pros:
- Contiguous memory (good cache locality for small/medium relation sets)

Cons:
- Vec::insert causes O(n) shifts for large sets

Cache Efficiency Score: 7/10

---

## 4) Binary Size (cargo bloat)

basic-app (release):
- .text size ~2.4 MiB, file size ~3.0 MiB
- top symbols dominated by sqlite/sqlx
- premix does not appear in top-N hotspots

relation-app (debug):
- same pattern, sqlx/sqlite dominate
- premix symbols are small and not top-N

Conclusion: no evidence of monomorphization bloat from Premix in examples.

---

## 5) Golden Ratio Comparison (Logic-to-Logic)

Premix path:
1) build SQL string
2) build binds Vec
3) render WHERE clause
4) bind values into query
5) execute

Raw sqlx path:
1) construct query
2) bind values
3) execute

Premix adds a few allocations and branches per query. Overhead is modest but
not zero.

---

## 6) Benchmark Comparison (premix_vs_sqlx)

From scripts/bench/bench_compare.ps1 (CPU core pinned, high priority), 3 runs:

Run 1:
```
raw_sqlx_fetch_one      time:   [55.894 us 56.231 us 56.602 us]
premix_find_fetch_one   time:   [66.175 us 66.704 us 67.280 us]
premix_static_query     time:   [61.972 us 62.249 us 62.522 us]
```

Run 2:
```
raw_sqlx_fetch_one      time:   [55.815 us 56.124 us 56.452 us]
premix_find_fetch_one   time:   [62.123 us 62.433 us 62.737 us]
premix_static_query     time:   [58.358 us 58.797 us 59.266 us]
```

Run 3:
```
raw_sqlx_fetch_one      time:   [55.355 us 55.561 us 55.772 us]
premix_find_fetch_one   time:   [64.447 us 65.375 us 66.297 us]
premix_static_query     time:   [60.148 us 60.515 us 60.901 us]
```

Median-of-medians:
- raw_sqlx_fetch_one: ~55.972 us
- premix_find_fetch_one: ~64.837 us
- premix_static_query: ~60.520 us

Relative (median-of-medians vs raw):
- premix_static_query: ~+8.1%
- premix_find_fetch_one: ~+15.8%

Interpretation: static query stays close to raw sqlx; dynamic query builder
adds ~16% on this sample set.

---
## 7) Recommendations

Low-risk optimizations:
- Replace column/op Strings with Cow<'static, str> or enums (reduce allocs).
- Use adaptive eager loading: Vec for small N, HashMap for large N.
- Expose an opt-in stream API without BoxStream for hot paths.

---

## Appendix: Evidence Snippets

Query builder entry points (line refs):
- filter_eq: premix-core/src/query.rs:236
- include: premix-core/src/query.rs:409
- to_sql: premix-core/src/query.rs:428
- stream: premix-core/src/query.rs:676

Eager loading layout:
- premix-macros/src/relations.rs: 107-188

---

## Status

This audit is based on static analysis + cargo expand + cargo bloat.
No runtime profiler (perf/dhat) was used in this pass.

For production decisions, pair this report with the benchmark suite in
benchmarks/ and docs/BENCHMARK_RESULTS.md.
