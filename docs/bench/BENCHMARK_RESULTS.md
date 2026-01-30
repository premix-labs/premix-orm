# Premix ORM Benchmark Results

**Date:** 2026-01-30 (Latest Run)  
**Environment:** Windows 11, Rust 2024 Edition  
**Test:** In-Memory SQLite + Real Postgres

---

## 5-Way Comparison (Standard CRUD)

| Operation | SQLx Raw | Premix ORM | SeaORM | Rbatis CRUD | Rbatis Raw |
|-----------|----------|------------|--------|-------------|------------|
| **Insert (1 row)** | 56.099 us | **57.213 us** | 62.512 us | 34.468 us | 32.742 us |
| **Select (1 row)** | 54.346 us | **54.154 us** | 60.246 us | 34.177 us | 33.239 us |
| **Bulk Select (100)** | 246.51 us | **225.18 us** | 1.045 ms | 1.170 ms | -- |

> **Premix vs The World:**
> - **Single Row:** Premix remains close to raw SQL and beats SeaORM; Rbatis raw is fastest in this run.
> - **Bulk Select:** Premix manual map slightly beats raw SQL; SeaORM and Rbatis are slower here.
> - **Notes:** Rbatis numbers include both CRUD and raw SQL paths where available.

---

## Relation Benchmark

### Case A: Single Relation (1 User -> 10 Posts)
| Method | Strategy | Time | Verdict |
|--------|----------|------|---------|
| **Raw SQL JOIN** | `JOIN` | **67.762 us** | Baseline |
| **Premix (Relation)** | Lazy | 122.84 us | +81% |
| **Premix (Eager)** | Batch | 124.73 us | +84% |
| **SeaORM** | Relation | 95.93 us | +42% |
| **Rbatis** | Manual | 66.295 us | -2% |

### Case B: Bulk Relation (50 Users -> 500 Posts)
| Method | Strategy | Time | vs Raw |
|--------|----------|------|--------|
| **Raw SQL JOIN** | `JOIN` | **10.354 ms** | Baseline |
| **Rbatis** | Manual Batch | 6.734 ms | -35% |
| **Premix (Eager)** | Batching | 2.969 ms | -71% |
| **SeaORM** | Loader | 9.067 ms | -12% |
| **Lazy Loading** | N+1 | 3.720 ms | -64% |

> **Note:** N+1 can look faster in micro-benchmarks but is typically worse at scale. Treat it as a baseline, not a recommended strategy.

---

## Modify Operations

| Operation | SQLx Raw | Premix | Rbatis Raw | SeaORM |
|-----------|----------|--------|------------|--------|
| **Update** | 109.08 us | **113.57 us** | 70.159 us | 245.47 us |
| **Delete (Hard)** | 113.45 us | **113.41 us** | 61.39 us | 131.85 us |

> Premix's update is ~2.3x faster than SeaORM in this run.

---

## Soft Delete

| Library | Raw SQL (UPDATE) | Premix (.delete()) |
|---------|------------------|--------------------|
| **Time** | **113.86 us** | 112.37 us |

---

## Transactions

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQLx** | 90.732 us | Baseline |
| **Premix** | 91.631 us | +1% |
| **Rbatis** | **47.166 us** | -48% |
| **SeaORM** | 91.606 us | +1% |

---

## Optimistic Locking

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQL** | 115.74 us | Baseline |
| **Premix** | 116.35 us | +1% |

---

## Bulk Operations (1000 Rows)

| Method | Time | Speedup |
|--------|------|---------|
| Loop Update (1 by 1) | 67.200 ms | Baseline |
| **Bulk Update** | **93.577 us** | **718x faster** |
> Speedup is vs the loop update shown above (same app, same dataset).

---

## I/O Performance (Real Postgres)

**Environment:** Localhost Postgres 18 (TCP connection)

### INSERT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **100.93 us** | Baseline |
| Premix | 101.51 us | +1% |
| SeaORM | 113.95 us | +13% |
| Rbatis | **95.637 us** | -5% |

### SELECT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | 66.038 us | Baseline |
| Premix | 61.962 us | -6% |
| SeaORM | 65.014 us | -2% |
| Rbatis | **55.397 us** | -16% |

### Concurrency (10 Parallel Selects)
| Method | Time |
|--------|------|
| **Raw SQL** | 644.09 us |
| Premix | **606.53 us** |

> In this run, Premix is slightly faster on select/concurrency; raw SQL still leads insert.

---

## Summary

| Metric | Premix vs SeaORM | Premix vs Rbatis Raw | Premix vs Raw SQL |
|--------|------------------|----------------------|-------------------|
| **Insert** | **1.1x faster** | **0.6x slower** | ~+2% |
| **Select** | **1.1x faster** | **0.6x slower** | ~0% |
| **Update** | **2.2x faster** | **0.6x slower** | ~+4% |
| **Transaction** | **1.0x** | **0.5x slower** | ~+1% |

---

## Methodology Notes (This Run)

- **Hardware/OS:** Windows 11, Intel Core i9-13900K (24C/32T), 32 GB RAM.
- **Bench framework:** Criterion (async_tokio), warmup 5s, measurement 10s, default sample size 50.
- **SQLite:** in-memory for CRUD + relations benchmarks (no real I/O latency). Raw SQL uses `query_as` to include decode overhead.
- **Postgres:** local Postgres 18 over TCP, no WAN latency.
- **Postgres settings:** shared_buffers = 16 MB, work_mem = 4 MB, effective_cache_size = 512 MB, max_connections = 100.
- **Postgres pool size:** 20 connections (PgPoolOptions::max_connections).
- **Dataset sizes:** See table titles (e.g., 1 row, 100 rows, 50 users -> 500 posts).
- **Criterion samples:** IO Insert = 50, IO Select = 100, IO Concurrency = 20, Bulk Ops = 10 (others default).
- **Premix paths:** Benchmarks use ultra/static paths where available (`save_ultra`/`update_ultra`/`delete_ultra`, `premix_query!`).
- **Rbatis paths:** Benchmarks include both CRUD macros and raw SQL (`exec`/`query_decode`) where applicable.
- **Value source:** Median values from `bench_orm_output.txt` and `bench_io_output.txt` (single run; raw SQL uses `query_as`).
- **Commands:** `scripts/bench/bench_orm.ps1`, `scripts/bench/bench_io.ps1`.
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
