# Introduction

Premix ORM is a zero-overhead, type-safe ORM for Rust. Its macros generate
query-building code at compile time, and the SQL strings are assembled at
runtime to stay as close to raw `sqlx` calls as possible.

> Research status: This project is a research prototype. APIs may change and
> production use is not recommended yet. This codebase is AI-assisted and
> should be treated as experimental.

## What Premix Optimizes For

- **Predictability:** You should be able to reason about the SQL that will run.
- **Performance:** The runtime path should be near raw `sqlx`.
- **Rust-First Modeling:** Your code should look like how you think about data.

## What This Book Covers

- Modeling data with `#[derive(Model)]`.
- Querying, filtering, and eager-loading relations.
- Schema management: `Premix::sync` and CLI migrations.
- Transactions, validation, and hooks.
- Multi-database setup and production considerations.

## What This Book Does Not Cover

- Full SQLx tutorial (connection pool setup, advanced SQL features).
- Advanced schema design (custom column types, indexes, constraints).
- A full web framework guide (Axum/Actix integration in detail).

## Prerequisites

- Rust 1.85+ with `cargo`.
- Familiarity with async Rust and `tokio`.
- Basic SQL knowledge (SELECT/INSERT/UPDATE/DELETE).

## Example Validation

This book's code samples are compile-checked through the dedicated
`orm-book/book-examples` test crate. Run:

```bash
cargo test
```

from `premix-orm/orm-book/book-examples` to validate examples.

## How to Read This Book

If you are new to Premix, start with "Getting Started". If you already know
the basics, jump to the chapters you need:

- **Models** for schema mapping.
- **Queries** for the query builder and raw SQL escape hatches.
- **Relations** for N+1 avoidance and eager loading.
- **Migrations** for schema changes.

For API surface details, prefer rustdoc (`cargo doc`) and the crate READMEs.
