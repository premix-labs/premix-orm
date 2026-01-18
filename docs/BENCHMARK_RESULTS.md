# Premix ORM Benchmark Results 🏆

**Date:** 2026-01-18 (Latest Run)  
**Environment:** Windows 11, Rust 2024 Edition  
**Test:** In-Memory SQLite + Real Postgres

---

## 📊 4-Way Comparison (Standard CRUD)

| Operation | SQLx Raw | Premix ORM | SeaORM | Rbatis |
|-----------|----------|------------|--------|--------|
| **Insert (1 row)** | 22.8 µs | **15.1 µs** | 29.7 µs | 17.7 µs |
| **Select (1 row)** | 12.2 µs | **12.3 µs** | 30.9 µs | 16.0 µs |
| **Bulk Select (100)** | 92.7 µs | 104.1 µs | 102.2 µs | 115.5 µs |

> [!NOTE]
> **Premix vs The World:**
> - **Single Row:** Premix is **faster than raw SQL** and 2x faster than SeaORM.
> - **Bulk Select:** Premix is competitive with dedicated ORMs.

---

## 🔗 Relation Benchmark

### Case A: Single Relation (1 User -> 10 Posts)
| Method | Strategy | Time | Verdict |
|--------|----------|------|---------|
| **Raw SQL JOIN** | `JOIN` | **24.9 µs** | Baseline |
| **Premix (Relation)** | Lazy | **36.4 µs** | +46% |
| **Premix (Eager)** | Batch | **36.8 µs** | +48% |
| **SeaORM** | Relation | **40.8 µs** | +63% |
| **Rbatis** | Manual | **70.3 µs** | +182% |

### Case B: Bulk Relation (50 Users -> 500 Posts) 🏆
| Method | Strategy | Time | vs Raw |
|--------|----------|------|--------|
| **Raw SQL JOIN** | `JOIN` | **689.9 µs** | Baseline |
| **Rbatis** | Manual Batch | **767.3 µs** | +11% |
| **Premix (Eager)** | Batching | **848.2 µs** | +23% |
| **SeaORM** | Loader | **1,030 µs** | +49% |
| **Lazy Loading** | N+1 | **2,311 µs** | +235% ❌ |

---

## 🛠️ Modify Operations

| Operation | SQLx Raw | Premix | Rbatis | SeaORM |
|-----------|----------|--------|--------|--------|
| **Update** | 25.9 µs | **29.4 µs** | 32.1 µs | 117.7 µs |
| **Delete (Hard)** | 25.8 µs | **26.6 µs** | 32.9 µs | 59.1 µs |

> [!NOTE]
> Premix's update is **~4.0x faster** than SeaORM!

---

## 🗑️ Soft Delete [New!]

| Library | Raw SQL (UPDATE) | Premix (.delete()) |
|---------|------------------|---------------------|
| **Time** | 28.4 µs | **30.9 µs** |

---

## 🏦 Transactions

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQLx** | 17.6 µs | Baseline |
| **Premix** | **17.7 µs** | +0.5% |
| **Rbatis** | 25.4 µs | +44% |
| **SeaORM** | 39.8 µs | +126% |

---

## 🔒 Optimistic Locking

| Library | Time | vs Raw |
|---------|------|--------|
| **Raw SQL** | 26.5 µs | Baseline |
| **Premix** | **30.2 µs** | +14% |

---

## 🚚 Bulk Operations (1000 Rows)

| Method | Time | Speedup |
|--------|------|---------|
| Loop Update (1 by 1) | 32.6 ms | Baseline |
| **Bulk Update** | **66.5 µs** | ⚡ **490x faster!** |

---

## 📡 I/O Performance (Real Postgres)

**Environment:** Localhost Postgres 18 (TCP connection)

### INSERT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Premix** | **120.6 µs** | ⚡ **~Same/Faster!** |
| Raw SQL | 121.9 µs | Baseline |
| SeaORM | 131.1 µs | +8% |
| Rbatis | 151.1 µs | +24% |

### SELECT (Real Network Latency)
| ORM | Time | vs Raw SQL |
|-----|------|------------|
| **Premix** | **61.5 µs** | ⚡ **Faster!** |
| Raw SQL | 65.2 µs | Baseline |
| SeaORM | 71.9 µs | +10% |
| Rbatis | 80.3 µs | +23% |

### Concurrency (10 Parallel Selects)
| Method | Time |
|--------|------|
| Raw SQL | 192.0 µs |
| **Premix** | 192.7 µs |

---

## Summary 🎯

| Metric | Premix vs SeaORM | Premix vs Rbatis | Premix vs Raw SQL |
|--------|------------------|------------------|-------------------|
| **Insert** | **1.9x faster** | **1.1x faster** | ⚡ **Faster!** |
| **Select** | **2.5x faster** | **1.3x faster** | ⚡ **~Same** |
| **Update** | **4.0x faster** | **1.1x faster** | ~Same |
| **Transaction** | **2.2x faster** | **1.4x faster** | ~Same |

---

## Versions Tested

```toml
tokio = "1.49.0"
sqlx = "0.8.6"
criterion = "0.8.1"
sea-orm = "1.1.19"
rbatis = "4.6.15"
chrono = "0.4.43"
```
