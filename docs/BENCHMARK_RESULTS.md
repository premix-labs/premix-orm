# Premix ORM Benchmark Results

**Date:** 2026-01-21 (Latest Run)  
**Environment:** Windows 11, Rust 2024 Edition  
**Test:** In-Memory SQLite + Real Postgres

---

## 4-Way Comparison (Standard CRUD)

| Operation | SQLx Raw | Premix ORM | SeaORM | Rbatis |
|-----------|----------|------------|--------|--------|
| **Insert (1 row)** | **11.74 us** | 12.34 us | 26.97 us | 14.99 us |
| **Select (1 row)** | 11.25 us | **11.16 us** | 19.83 us | 14.49 us |
| **Bulk Select (100)** | **70.56 us** | 94.04 us | 97.42 us | 96.23 us |

> **Premix vs The World:**
> - **Single Row:** Premix is faster than SeaORM/Rbatis and within ~1% of raw SQL.
> - **Bulk Select:** Premix is competitive with other ORMs, but raw SQL remains fastest here.

---

## Relation Benchmark

### Case A: Single Relation (1 User -> 10 Posts)
| Method | Strategy | Time | Verdict |
|--------|----------|------|---------|
| **Raw SQL JOIN** | `JOIN` | **24.48 us** | Baseline |
| **Premix (Relation)** | Lazy | 33.20 us | +36% |
| **Premix (Eager)** | Batch | 34.36 us | +40% |
| **SeaORM** | Relation | 37.47 us | +53% |
| **Rbatis** | Manual | 41.27 us | +69% |

### Case B: Bulk Relation (50 Users -> 500 Posts)
| Method | Strategy | Time | vs Raw |
|--------|----------|------|--------|
| **Raw SQL JOIN** | `JOIN` | **526.94 us** | Baseline |
| **Rbatis** | Manual Batch | 541.43 us | +3% |
| **Premix (Eager)** | Batching | 630.17 us | +20% |
| **SeaORM** | Loader | 845.48 us | +60% |
| **Lazy Loading** | N+1 | 1.51 ms | +187% |

---

## Modify Operations

| Operation | SQLx Raw | Premix | Rbatis | SeaORM |
|-----------|----------|--------|--------|--------|
| **Update** | **23.73 us** | 25.36 us | 27.95 us | 110.34 us |
| **Delete (Hard)** | **22.96 us** | 24.30 us | 27.59 us | 54.91 us |

> Premix's update is ~3.1x faster than SeaORM.

---

## Soft Delete

| Library | Raw SQL (UPDATE) | Premix (.delete()) |
|---------|------------------|--------------------|
| **Time** | **24.37 us** | 24.81 us |

---

## Transactions

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQLx** | **14.83 us** | Baseline |
| **Premix** | 14.90 us | +1% |
| **Rbatis** | 20.39 us | +37% |
| **SeaORM** | 33.63 us | +127% |

---

## Optimistic Locking

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQL** | **24.74 us** | Baseline |
| **Premix** | 26.32 us | +6% |

---

## Bulk Operations (1000 Rows)

| Method | Time | Speedup |
|--------|------|---------|
| Loop Update (1 by 1) | 13.38 ms | Baseline |
| **Bulk Update** | **55.40 us** | **241x faster** |
> Speedup is vs the loop update shown above (same app, same dataset).

---

## I/O Performance (Real Postgres)

**Environment:** Localhost Postgres 18 (TCP connection)

### INSERT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **102.79 us** | Baseline |
| Premix | 106.41 us | +4% |
| SeaORM | 107.86 us | +5% |
| Rbatis | 99.54 us | -3% |

### SELECT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **54.68 us** | Baseline |
| Premix | 54.49 us | -0% |
| SeaORM | 63.21 us | +16% |
| Rbatis | 55.74 us | +2% |

### Concurrency (10 Parallel Selects)
| Method | Time |
|--------|------|
| **Raw SQL** | **178.66 us** |
| Premix | 179.37 us |

> In this run, raw SQL leads select/concurrency while Premix remains close.

---

## Summary

| Metric | Premix vs SeaORM | Premix vs Rbatis | Premix vs Raw SQL |
|--------|------------------|------------------|-------------------|
| **Insert** | **2.2x faster** | **1.2x faster** | ~Same |
| **Select** | **1.8x faster** | **1.3x faster** | ~Same |
| **Update** | **4.4x faster** | **1.1x faster** | ~Same |
| **Transaction** | **2.3x faster** | **1.4x faster** | ~Same |

---

## Methodology Notes (This Run)

- **Hardware/OS:** Windows 11, Intel Core i9-13900K (24C/32T), 32 GB RAM.
- **Bench framework:** Criterion (async_tokio), warmup 3s, measurement 10s, default sample size 50.
- **SQLite:** in-memory for CRUD + relations benchmarks (no real I/O latency).
- **Postgres:** local Postgres 18 over TCP, no WAN latency.
- **Postgres settings:** shared_buffers = 16 MB, work_mem = 4 MB, effective_cache_size = 512 MB, max_connections = 100.
- **Postgres pool size:** 20 connections (PgPoolOptions::max_connections).
- **Dataset sizes:** See table titles (e.g., 1 row, 100 rows, 50 users -> 500 posts).
- **Criterion samples:** IO Insert = 50, IO Select = 100, IO Concurrency = 20, Bulk Ops = 10 (others default).
- **Value source:** Median of medians across 3 rounds, from `benchmarks/results/summary.csv`.
- **Commands:** `scripts/bench/bench_repeat.ps1 -Rounds 3`.
- **Commit:** 71d26aad0e3ef4798296e2bb62113d677b320349
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
