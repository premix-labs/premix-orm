# Master Flowplan: Premix ORM

## Project Concept
**Concept:** The "Holy Grail" of Rust ORMs.
**Slogan:** "Write Rust, run optimized SQL."

## 1. Core Philosophy (5 Pillars of a Great ORM)

### 1. The "Zero-Overhead" Abstraction (Fastest)
> "Your code should be as fast as handwriting raw SQL."
- **Premix Approach:** Uses Rust macros to generate SQL at compile time. At runtime it executes pre-prepared SQL.
- **Goal:** Runtime benchmarks should match raw `sqlx` (0% overhead).

### 2. The "Mental Model Match" (Easiest)
> "Code should look like the way you think, not the way the database stores."
- **Premix Approach:** Connects the object world (`User`) with the table world (`users`).
  - **Naming Convention:** `User` -> `users` automatically.
  - **Active Record:** `user.save()`, `User::find(1)`.
  - **Auto Migration:** Change a struct, update the database.

### 3. The "Impossible-to-Fail" Safety (Most Stable)
> "If it compiles, it runs without SQL errors."
- **Premix Approach:** Move errors to compile time.
  - Wrong field name -> compile error.
  - Type mismatch -> compile error.
- **Goal:** Eliminate runtime surprises.

### 4. The "Glass Box" Transparency (Transparent)
> "Magic is good, but Black Magic is bad."
- **Premix Approach:** Expose generated SQL via `to_sql()` or similar helpers.
- **Goal:** Developers should feel in control of the SQL.

### 5. The "Graceful Escape Hatch" (Flexible)
> "Easy things should be easy, hard things should be possible."
- **Premix Approach:** Allow mixing raw SQL with the ORM for complex cases.
- **Example:** `User::raw_sql("SELECT * FROM ...").fetch_all()`

---

## 2. Development Flowplan

### Phase 0: Setup and Architecture - COMPLETED
**Mission:** Lay the foundation for a scalable project structure.

- [x] Project initialization (Cargo workspace).
- [x] Module separation (`premix-core` runtime, `premix-macros` compiler).
- [x] Basic `#[derive(Model)]` macro.
- [x] Table name generation (`User` -> `users`).
- [x] Column extraction from struct fields.

**Tech Stack:**
- Database driver: `sqlx`
- Macro engine: `syn`, `quote`, `proc-macro2`
- Runtime: `tokio`

### Phase 1: The CRUD Engine - COMPLETED
**Mission:** Basic persistence and querying.

- [x] Type mapping system (`i32` -> `INTEGER`, `String` -> `TEXT`).
- [x] Connection integration with `sqlx`.

**Milestone 1:** `User::new().save().await` persists data to a real database.

### Phase 2: The Migration Magic - COMPLETED
**Mission:** Avoid manual SQL for schema changes.

- [x] Schema introspection.
- [x] Diff engine between structs and database.
- [x] `Premix::sync()` capability to run `CREATE TABLE` or `ALTER TABLE`.

**Milestone 2:** Add a field to a struct, run sync, and the database updates.

### Phase 3: Relations and Optimization - COMPLETED
**Mission:** Solve N+1 and provide fluent queries.

- [x] Relation macros: `#[has_many]`, `#[belongs_to]`.
- [x] Query builder: `Model::find()`, `.filter()`, `.limit()`, `.offset()`.
- [x] Eager loading with `.include("posts")`.
- [x] `#[premix(ignore)]` for non-column fields.

**Milestone 3:** Fetch 100 users and posts without N+1 queries.

### Phase 4: Developer Experience - COMPLETED
**Mission:** Make the ORM usable in real projects.

- [x] CLI tool (`premix-cli`).
- [x] Documentation and examples.
- [x] Macro error handling with `syn::Error`.

**Milestone 4:** Full ecosystem for both runtime and DX.

### Phase 5: Enterprise Standard - COMPLETED
**Mission:** Support real-world complexity.

- [x] Observability via `tracing`.
- [x] ACID transactions with `pool.begin()`.
- [x] Lifecycle hooks (`before_save`, `after_save`).
- [x] Optimistic locking via `update()`.
- [x] Validation via `validate()`.

**Milestone 5:** Ready for large-scale systems.

### Phase 6: The Versatility - COMPLETED
**Mission:** Remove limitations and support multiple databases.

- [x] Multi-database architecture via `SqlDialect`.
- [x] Generic `Model<DB>` and `QueryBuilder<DB>`.
- [x] SQLite, PostgreSQL, MySQL support (feature-gated).
- [x] Soft deletes (`deleted_at`).
- [x] Bulk operations (`QueryBuilder::update`, `QueryBuilder::delete`).
- [x] JSON/JSONB support via `serde_json`.

**Milestone 6:** Multi-DB support and bulk ops in place.

### Phase 7: DevOps (Versioned Migrations) - COMPLETED
**Mission:** Support team workflows and release readiness.

- [x] `premix-cli migrate` command family.
- [x] `create`, `up` migrations.
- [x] Migration tracking via `_premix_migrations`.

**Milestone 7:** Production migration system ready.

### Phase 8: The Scale - PLANNED
**Mission:** High availability for large systems.

- [ ] Read/write splitting (primary + replicas).
- [ ] Connection resolver for multi-tenancy.

### Phase 9: Advanced Relations - DEFERRED
**Mission:** Support advanced/niche modeling.

- [ ] Polymorphic relations.
- [ ] Declarative schema definitions.

### Phase 10: Legacy Support - PLANNED
**Mission:** Support brownfield projects.

- [ ] Composite primary keys.
- [ ] Custom Postgres types and domains.

### Optional: Futurism - PLANNED
**Mission:** Prepare for long-term trends.

- [ ] Vector types and semantic search.
- [ ] Edge/Wasm targets.
- [ ] Local-first sync (CRDTs).
- [ ] Adaptive self-optimization.

---

## 3. Developer Automation Suite

Scripts live under `scripts/`:

### `scripts/dev` (Daily Development)
- `run_fmt.ps1`: Format code and fix clippy warnings.
- `run_clean.ps1`: Clean build artifacts and database files.
- `gen_docs.ps1`: Generate rustdoc and mdBook.

### `scripts/test` (Verification)
- `test_quick.ps1`: Smoke test (build + run basic app).
- `test_examples.ps1`: Run all example apps.
- `test_migration.ps1`: E2E test for migrations.

### `scripts/ci` (Quality Assurance)
- `check_all.ps1`: Build, test, clippy, format.
- `check_audit.ps1`: Security scan via `cargo audit`.
- `check_coverage.ps1`: Code coverage via `cargo tarpaulin`.

### `scripts/bench` (Performance)
- `bench_orm.ps1`: SQLite benchmark vs other ORMs.
- `bench_io.ps1`: Postgres I/O benchmark.

### `scripts/release` (Deployment)
- `run_publish.ps1`: Publish to crates.io.

---

## 4. Architecture Diagram (Mental Model)

### Working Flow
1. **Compile time:** User writes Rust -> macros generate optimized SQL.
2. **Runtime:** App executes generated SQL through `sqlx`.

---

## 5. Engineering Risks

### 1. Compile Time Explosion
- **Risk:** Heavy macros slow builds.
- **Solution:** Keep codegen lean and support incremental compilation.

### 2. Error Message Complexity
- **Risk:** Macro errors point to the wrong place.
- **Solution:** Use `syn::spanned` to target errors precisely.

### 3. Async in Traits
- **Risk:** Rust limitations around async trait bounds.
- **Solution:** Carefully design ownership, use BoxFutures when needed.
