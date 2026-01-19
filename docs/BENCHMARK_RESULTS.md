# Premix ORM Benchmark Results

**Date:** 2026-01-19 (Latest Run)  
**Environment:** Windows 11, Rust 2024 Edition  
**Test:** In-Memory SQLite + Real Postgres

---

## 4-Way Comparison (Standard CRUD)

| Operation | SQLx Raw | Premix ORM | SeaORM | Rbatis |
|-----------|----------|------------|--------|--------|
| **Insert (1 row)** | **25.9 us** | **25.9 us** | 39.4 us | 30.6 us |
| **Select (1 row)** | **25.2 us** | 25.4 us | 42.9 us | 33.7 us |
| **Bulk Select (100)** | **84.7 us** | 101.1 us | 105.2 us | 107.0 us |

> **Premix vs The World:**
> - **Single Row:** Premix is faster than SeaORM/Rbatis and within ~1% of raw SQL.
> - **Bulk Select:** Premix is competitive with other ORMs, but raw SQL remains fastest here.

---

## Relation Benchmark

### Case A: Single Relation (1 User -> 10 Posts)
| Method | Strategy | Time | Verdict |
|--------|----------|------|---------|
| **Raw SQL JOIN** | `JOIN` | **33.6 us** | Baseline |
| **Premix (Relation)** | Lazy | **56.8 us** | +69% |
| **Premix (Eager)** | Batch | **57.5 us** | +71% |
| **SeaORM** | Relation | **47.3 us** | +41% |
| **Rbatis** | Manual | **78.1 us** | +133% |

### Case B: Bulk Relation (50 Users -> 500 Posts)
| Method | Strategy | Time | vs Raw |
|--------|----------|------|--------|
| **Raw SQL JOIN** | `JOIN` | **557.6 us** | Baseline |
| **Rbatis** | Manual Batch | **585.5 us** | +5% |
| **Premix (Eager)** | Batching | **682.9 us** | +22% |
| **SeaORM** | Loader | **858.0 us** | +54% |
| **Lazy Loading** | N+1 | **1.9 ms** | +241% |

---

## Modify Operations

| Operation | SQLx Raw | Premix | Rbatis | SeaORM |
|-----------|----------|--------|--------|--------|
| **Update** | **52.1 us** | 53.1 us | 60.5 us | 165.3 us |
| **Delete (Hard)** | 55.8 us | **53.0 us** | 61.1 us | 79.6 us |

> Premix's update is ~3.1x faster than SeaORM.

---

## Soft Delete

| Library | Raw SQL (UPDATE) | Premix (.delete()) |
|---------|------------------|--------------------|
| **Time** | **53.2 us** | 53.5 us |

---

## Transactions

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQLx** | **46.7 us** | Baseline |
| **Premix** | 47.0 us | +1% |
| **Rbatis** | 58.6 us | +26% |
| **SeaORM** | 68.0 us | +46% |

---

## Optimistic Locking

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQL** | **53.6 us** | Baseline |
| **Premix** | 54.4 us | +1% |

---

## Bulk Operations (1000 Rows)

| Method | Time | Speedup |
|--------|------|---------|
| Loop Update (1 by 1) | 27.2 ms | Baseline |
| **Bulk Update** | **62.9 us** | **432x faster** |
> Speedup is vs the loop update shown above (same app, same dataset).

---

## I/O Performance (Real Postgres)

**Environment:** Localhost Postgres 18 (TCP connection)

### INSERT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **102.5 us** | Baseline |
| Premix | 106.5 us | +4% |
| SeaORM | 109.2 us | +7% |
| Rbatis | 105.2 us | +3% |

### SELECT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Raw SQL** | **53.1 us** | Baseline |
| Premix | 56.6 us | +7% |
| SeaORM | 61.9 us | +17% |
| Rbatis | 57.1 us | +8% |

### Concurrency (10 Parallel Selects)
| Method | Time |
|--------|------|
| **Raw SQL** | **185.3 us** |
| Premix | 188.4 us |

> In this run, raw SQL leads select/concurrency while Premix remains close.

---

## Summary

| Metric | Premix vs SeaORM | Premix vs Rbatis | Premix vs Raw SQL |
|--------|------------------|------------------|-------------------|
| **Insert** | **1.5x faster** | **1.2x faster** | ~Same |
| **Select** | **1.7x faster** | **1.3x faster** | ~Same |
| **Update** | **3.1x faster** | **1.1x faster** | ~Same |
| **Transaction** | **1.4x faster** | **1.2x faster** | ~Same |

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
- **Value source:** Median of medians across 3 rounds, from `benchmarks/results/summary.csv` (Criterion `base/estimates.json`).
- **Commands:** `scripts/bench/bench_repeat.ps1 -Rounds 3`.
- **Commit:** 012117dd573d72c8772a7cba17e9a862b34a0520
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
