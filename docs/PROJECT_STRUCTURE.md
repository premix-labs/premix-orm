# Project Structure

This document summarizes the purpose of the main folders and crates in the
Premix ORM repository.

## Top-Level Crates

- `premix-core/`: Runtime library (models, query builder, migrations, execution).
- `premix-macros/`: Procedural macros (`#[derive(Model)]`, relations, codegen).
- `premix-orm/`: Facade crate that re-exports core + macros for end users.
- `premix-cli/`: CLI tooling for init, sync, and migrations.

## Examples

Each example is a small, runnable app focused on one feature area.

- `examples/basic-app/`: Minimal CRUD example.
- `examples/axum-app/`: Lightweight Axum integration example.
- `examples/premix-axum-tutorial/`: Full Axum tutorial (API + DB workflow).
- `examples/relation-app/`: Relations (`has_many`, `belongs_to`).
- `examples/eager-app/`: Eager loading to avoid N+1.
- `examples/bulk-ops-app/`: Bulk update/delete examples.
- `examples/hooks-app/`: Lifecycle hooks (`before_save`, `after_save`).
- `examples/optimistic-locking-app/`: Optimistic locking behavior.
- `examples/soft-delete-app/`: Soft delete flows.
- `examples/tracing-app/`: Tracing/observability setup.
- `examples/transaction-app/`: Transaction usage.
- `examples/validation-app/`: Model validation examples.

## Documentation

- `README.md`: Project overview and quick start.
- `docs/`: Architecture, development flowplan, benchmarks, testing, philosophy.
- `orm-book/`: mdBook long-form guide and compile-checked examples.
- `DISCLAIMER.md`: Research/alpha status notice.
- `RELEASE_CHECKLIST.md`: Release steps and sanity checks.

## Scripts

- `scripts/dev/`: Format, clean, and doc generation.
- `scripts/test/`: Smoke tests and E2E checks.
- `scripts/ci/`: CI-style checks (build, test, audit, coverage).
- `scripts/bench/`: Benchmark drivers and repeat runs.
- `scripts/release/`: Publish automation.

## Benchmarks and Results

- `benchmarks/`: Criterion benches and comparison code.
- `benchmarks/results/`: Stored benchmark artifacts and summaries.

## Assets and Data

- `assets/`: Logos, banners, and diagrams.
- `migrations/`: Versioned SQL migration files.

## Generated Output (not source)

- `target/`: Cargo build output.
- `coverage/`: Tarpaulin HTML report output.
