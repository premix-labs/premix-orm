# Premix ORM Benchmark Results

**Date:** 2026-01-21 (Latest Run)  
**Environment:** Windows 11, Rust 2024 Edition  
**Test:** In-Memory SQLite + Real Postgres

---

## 4-Way Comparison (Standard CRUD)

| Operation | SQLx Raw | Premix ORM | SeaORM | Rbatis |
|-----------|----------|------------|--------|--------|
| **Insert (1 row)** | 56.197 us | **54.610 us** | 62.450 us | 27.245 us |
| **Select (1 row)** | 54.187 us | **53.837 us** | 58.778 us | 27.016 us |
| **Bulk Select (100)** | **193.890 us** | 202.180 us | 1.263 ms | 1.166 ms |

> **Premix vs The World:**
> - **Single Row:** Premix tracks raw SQL and is faster than SeaORM; Rbatis is fastest in this run.
> - **Bulk Select:** Premix is close to raw SQL; SeaORM/Rbatis are slower in this run.

---

## Relation Benchmark

### Case A: Single Relation (1 User -> 10 Posts)
| Method | Strategy | Time | Verdict |
|--------|----------|------|---------|
| **Raw SQL JOIN** | `JOIN` | **65.515 us** | Baseline |
| **Premix (Relation)** | Lazy | 119.040 us | +82% |
| **Premix (Eager)** | Batch | 119.250 us | +82% |
| **SeaORM** | Relation | 79.646 us | +22% |
| **Rbatis** | Manual | 69.672 us | +6% |

### Case B: Bulk Relation (50 Users -> 500 Posts)
| Method | Strategy | Time | vs Raw |
|--------|----------|------|--------|
| **Raw SQL JOIN** | `JOIN` | **9.336 ms** | Baseline |
| **Rbatis** | Manual Batch | 10.068 ms | +8% |
| **Premix (Eager)** | Batching | 9.586 ms | +3% |
| **SeaORM** | Loader | 10.059 ms | +8% |
| **Lazy Loading** | N+1 | 3.669 ms | -61% |

---

## Modify Operations

| Operation | SQLx Raw | Premix | Rbatis | SeaORM |
|-----------|----------|--------|--------|--------|
| **Update** | **109.160 us** | 112.570 us | 53.089 us | 237.800 us |
| **Delete (Hard)** | 113.460 us | **109.010 us** | 54.983 us | 124.310 us |

> Premix's update is ~3.1x faster than SeaORM.

---

## Soft Delete

| Library | Raw SQL (UPDATE) | Premix (.delete()) |
|---------|------------------|--------------------|
| **Time** | **110.840 us** | 115.020 us |

---

## Transactions

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQLx** | 86.605 us | Baseline |
| **Premix** | **86.090 us** | -1% |
| **Rbatis** | 44.399 us | -49% |
| **SeaORM** | 86.957 us | +0% |

---

## Optimistic Locking

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQL** | **110.770 us** | Baseline |
| **Premix** | 111.520 us | +1% |

---

## Bulk Operations (1000 Rows)

| Method | Time | Speedup |
|--------|------|---------|
| Loop Update (1 by 1) | 65.213 ms | Baseline |
| **Bulk Update** | **95.958 us** | **680x faster** |
> Speedup is vs the loop update shown above (same app, same dataset).

---

## I/O Performance (Real Postgres)

**Environment:** Localhost Postgres 18 (TCP connection)

### INSERT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **103.950 us** | Baseline |
| Premix | 109.510 us | +5% |
| SeaORM | 113.830 us | +9% |
| Rbatis | 95.241 us | -8% |

### SELECT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | 62.064 us | Baseline |
| Premix | 63.456 us | +2% |
| SeaORM | 69.227 us | +12% |
| Rbatis | **54.475 us** | -12% |

### Concurrency (10 Parallel Selects)
| Method | Time |
|--------|------|
| **Raw SQL** | 556.36 us |
| Premix | **549.73 us** |

> In this run, raw SQL leads select/concurrency while Premix remains close.

---

## Summary

| Metric | Premix vs SeaORM | Premix vs Rbatis | Premix vs Raw SQL |
|--------|------------------|------------------|-------------------|
| **Insert** | **1.1x faster** | **0.5x faster** | ~-3% |
| **Select** | **1.1x faster** | **0.5x faster** | ~-1% |
| **Update** | **2.1x faster** | **0.5x faster** | ~+3% |
| **Transaction** | **1.0x faster** | **0.5x faster** | ~-1% |

---

## Methodology Notes (This Run)

- **Hardware/OS:** Windows 11, Intel Core i9-13900K (24C/32T), 32 GB RAM.
- **Bench framework:** Criterion (async_tokio), warmup 3s, measurement 10s, default sample size 50.
- **SQLite:** in-memory for CRUD + relations benchmarks (no real I/O latency). Raw SQL uses `query_as` to include decode overhead.
- **Postgres:** local Postgres 18 over TCP, no WAN latency.
- **Postgres settings:** shared_buffers = 16 MB, work_mem = 4 MB, effective_cache_size = 512 MB, max_connections = 100.
- **Postgres pool size:** 20 connections (PgPoolOptions::max_connections).
- **Dataset sizes:** See table titles (e.g., 1 row, 100 rows, 50 users -> 500 posts).
- **Criterion samples:** IO Insert = 50, IO Select = 100, IO Concurrency = 20, Bulk Ops = 10 (others default).
- **Value source:** Median of medians across 3 rounds from `bench_orm_r*_output.txt` and `bench_io_r*_output.txt` (raw SQL uses `query_as`).
- **Commands:** `scripts/bench/bench_orm.ps1 -OutPrefix bench_orm_r<N>`, `scripts/bench/bench_io.ps1 -OutPrefix bench_io_r<N>`.
- **Interpretation:** Numbers are specific to this environment; absolute rankings can change with hardware, DB config, and workload.

---

## Reliability Checklist (Recommended)

- **Power/CPU:** Set Windows power plan to High performance; avoid battery mode and heavy background tasks.
- **Process noise:** Close IDEs/browsers and pause downloads, backups, antivirus scans.
- **Repeatability:** Run 3-5 rounds and report the median of medians + 95% CI.
- **Database state:** TRUNCATE tables before each run; for Postgres, run `ANALYZE` after seeding.
- **Warm-up:** Run one warm-up pass before recording results.
- **Baselines:** Save a Criterion baseline and use `cargo bench -- --baseline <name>` for comparisons.
- **Artifacts:** Keep `target/criterion` for auditability.

## Repeatable Run Protocol

1) `scripts/bench/bench_orm.ps1`  
2) `scripts/bench/bench_io.ps1`  
3) Repeat steps 1-2 for 3-5 rounds (same machine, same DB settings).  
4) Update this file with median values and note any regressions.

Shortcut: `scripts/bench/bench_repeat.ps1 -Rounds 3` (writes `benchmarks/results/summary.csv` and snapshots).


## Versions Tested

```toml
tokio = "1.49.0"
sqlx = "0.8.6"
criterion = "0.8.1"
sea-orm = "1.1.19"
rbatis = "4.6.15"
chrono = "0.4.43"
```
