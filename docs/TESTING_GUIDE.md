# 🧪 Premix ORM Testing Guide

Quick reference for running tests, examples, and benchmarks across the workspace.

## 1. 🏗️ Core & Basic Verification
Validate basic CRUD operations and Web integration.

| Scenario | Command | Description |
|----------|---------|-------------|
| **Core CRUD** | `cargo run -p basic-app` | Creates, Reads, Updates, Deletes users using `QueryBuilder`. |
| **Web API** | `cargo run -p axum-app` | Runs an Axum web server integration test. |

## 2. 🚀 Performance & Relations (Phase 4)
Verify N+1 solutions and Eager Loading.

| Scenario | Command | Description |
|----------|---------|-------------|
| **Lazy Loading** | `cargo run -p relation-app` | Standard 1-N relation fetching (Manual Loop / N+1). |
| **Eager Loading** | `cargo run -p eager-app` | Batch fetching logic (`.include("posts")`). |
| **Benchmark (All)** | `cargo bench` | Run all performance benchmarks (Regression Test). |
| **Benchmark (Specific)** | `cargo bench -p benchmarks -- "Bulk Select"` | Run only specific benchmark group. |

## 3. 🛡️ Enterprise Features (Phase 6)
Verify stability, safety, and observability features.

| Scenario | Command | Description |
|----------|---------|-------------|
| **Transactions** | `cargo run -p transaction-app` | ACID compliance test (Commit/Rollback). |
| **Hooks** | `cargo run -p hooks-app` | Verify `before_save` / `after_save` triggers. |
| **Observability** | `cargo run -p tracing-app` | Check structured logging output. |

## 4. 🔒 Data Integrity & Validation
Ensure data consistency features work as expected.

| Scenario | Command | Description |
|----------|---------|-------------|
| **Optimistic Locking** | `cargo run -p optimistic-locking-app` | Test concurrency checks (version conflict). |
| **Validation** | `cargo run -p validation-app` | Test `validate()` trait logic. |

## 5. 🗑️ Advanced Data Management (Phase 6.5+)
Verify specialized data handling operations.

| Scenario | Command | Description |
|----------|---------|-------------|
| **Soft Deletes** | `cargo run -p soft-delete-app` | Verify `deleted_at` behavior implementation. |
| **Bulk Ops** | `cargo run -p bulk-ops-app` | Test `update_all` and `delete_all` efficiency. |

## 🛠️ Internal Unit Tests
For deep core library verification.

```bash
cargo test -p premix-core
cargo test -p premix-macros
```
