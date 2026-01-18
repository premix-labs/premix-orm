# 🧪 Premix ORM Testing Guide

Quick reference for running tests, examples, and benchmarks across the workspace.

## 🐚 Helper Scripts (Recommended)

We provide categorized PowerShell scripts in `scripts/` to standardize testing:

| Command | Category | Description |
|---------|----------|-------------|
| `./scripts/test/test_quick.ps1` | Smoke Test | Fast build and run of `basic-app`. |
| `./scripts/test/test_examples.ps1` | Regression | Runs all example apps in sequence. |
| `./scripts/bench/bench_orm.ps1` | Perf | SQLite benchmark against SeaORM/Rbatis. |
| `./scripts/bench/bench_io.ps1` | Perf (I/O) | Heavy Postgres I/O benchmark. |
| `./scripts/ci/check_all.ps1` | CI | Full check (Build, Test, Clippy, Format). |

## 1. 🏗️ Core & Basic Verification

| Scenario | Command | Description |
|----------|---------|-------------|
| **Core CRUD** | `cargo run -p basic-app` | Creates, Reads, Updates, Deletes users. |
| **Tutorial App** | `cargo run -p premix-axum-tutorial` | E2E test for the official tutorial. |
| **Simple Axum** | `cargo run -p axum-app` | Basic web server integration. |

## 2. 🚀 Performance & Relations

| Scenario | Command | Description |
|----------|---------|-------------|
| **Lazy Loading** | `cargo run -p relation-app` | Standard 1-N relation fetching. |
| **Eager Loading** | `cargo run -p eager-app` | Batch fetching logic (`.include("posts")`). |
| **Benchmark (All)** | `cargo bench` | Run all performance benchmarks. |

## 3. 🛡️ Enterprise Features

| Scenario | Command | Description |
|----------|---------|-------------|
| **Transactions** | `cargo run -p transaction-app` | ACID compliance test (Commit/Rollback). |
| **Hooks** | `cargo run -p hooks-app` | Verify `before_save` / `after_save` triggers. |
| **Observability** | `cargo run -p tracing-app` | Check structured logging output. |

## 4. 🔒 Data Integrity & Validation

| Scenario | Command | Description |
|----------|---------|-------------|
| **Optimistic Locking** | `cargo run -p optimistic-locking-app` | Test concurrency checks (version conflict). |
| **Validation** | `cargo run -p validation-app` | Test `validate()` trait logic. |

## 5. 🗑️ Advanced Data Management

| Scenario | Command | Description |
|----------|---------|-------------|
| **Soft Deletes** | `cargo run -p soft-delete-app` | Verify `deleted_at` behavior. |
| **Bulk Ops** | `cargo run -p bulk-ops-app` | Test `update_all` and `delete_all`. |

## 🛠️ Internal Unit Tests

```bash
cargo test -p premix-core
cargo test -p premix-macros
```
