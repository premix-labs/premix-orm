# Master Flowplan: Premix ORM (Ultimate Edition)

## Project Concept
**Concept:** The "Holy Grail" of Rust ORMs.  
**Slogan:** "Write Rust, run optimized SQL."  
**Status:** Alpha / Pre-release (v0.x).

## 1. Core Philosophy (5 Pillars of a Great ORM)

### 1. The "Zero-Overhead" Abstraction (Fastest)
> "Your code should be as fast as handwriting raw SQL."
- **Premix Approach:** Compile-time SQL generation. Runtime executes pre-baked SQL via `sqlx`.
- **Optimization:** Smart configuration defaults to auto-tune connection pools based on environment (Server vs Serverless).

### 2. The "Mental Model Match" (Easiest)
> "Code should look like the way you think, not the way the database stores."
- **Premix Approach:** Seamless object-table mapping.
- **Goal:** Intuitive Active Record API and frictionless testing experience.

### 3. The "Impossible-to-Fail" Safety (Most Stable)
> "If it compiles, it runs without SQL errors."
- **Premix Approach:** Compile-time validation.
- **Safety Rails:** Destructive guards prevent accidental mass deletes without explicit confirmation.

### 4. The "Glass Box" Transparency (Transparent)
> "Magic is good, but Black Magic is bad."
- **Premix Approach:** Expose generated SQL via `to_sql()`.
- **Security:** Sensitive data masking in logs (`***`) for PII protection.

### 5. The "Graceful Escape Hatch" (Flexible)
> "Easy things should be easy, hard things should be possible."
- **Premix Approach:** Mix raw SQL smoothly.
- **Feature:** Arbitrary struct mapping for complex reporting queries.

---

## 2. Development Flowplan

### Phase 0: Setup and Architecture - ✅ COMPLETED
**Mission:** Lay the foundation for a scalable project structure.
- [x] Project initialization & module separation.
- [x] Basic `#[derive(Model)]` macro.

### Phase 1: The CRUD Engine - ✅ COMPLETED
**Mission:** Basic persistence and querying.
- [x] Type mapping system.
- [x] Connection integration.

### Phase 2: The Migration Magic - ✅ COMPLETED
**Mission:** Avoid manual SQL for schema changes.
- [x] Schema introspection & diff engine.
- [x] `Premix::sync()` capability.
  - SQLite v1: tables/columns/types/nullability/pk/indexes/foreign keys diff and SQL generation.
  - Postgres v1: tables/columns/types/nullability/pk/indexes/foreign keys diff and SQL generation.

### Phase 3: Relations and Optimization - ✅ COMPLETED
**Mission:** Solve N+1 and provide fluent queries.
- [x] Relation macros (`has_many`, `belongs_to`).
- [x] Eager loading (`.include()`) with O(1) query strategy.
  - Eager loading supports both has_many and belongs_to.

### Phase 4: Developer Experience (DX) - ✅ COMPLETED
**Mission:** Make the ORM usable, testable, and adoptable.
- [x] CLI tool (`premix-cli`).
- [x] Documentation and examples.
- [x] Macro error handling.
- [x] Glass Box documentation (macro expansion + SQL flow).
- [x] Performance tuning guide (prepared statements, fast/static paths).
- [x] Test Utilities (New):
  - [x] Transactional tests (auto-rollback after each test case).
  - [x] MockDatabase helper.
- [x] Database Scaffolding (New):
  - [x] `premix-cli scaffold`: Generate Rust structs from an existing database.
- [x] Framework Integrations (New):
  - [x] Official helpers: `premix-axum`, `premix-actix`.

### Phase 5: Enterprise Standard - ✅ COMPLETED
**Mission:** Support real-world complexity, security, and reporting.
- [x] Observability (`tracing`).
- [x] ACID transactions & lifecycle hooks.
- [x] Optimistic locking & validation.
- [x] Arbitrary Struct Mapping (New):
  - [x] `Premix::raw("...").fetch_as::<ReportStruct>()`.
- [x] Sensitive Data Masking (New):
  - [x] `#[premix(sensitive)]` attribute to redact data in logs.
- [x] Smart Configuration (New):
  - [x] Auto-tune pool settings based on environment detection.

### Phase 6: The Versatility - ✅ COMPLETED
**Mission:** Remove limitations and ensure safety.
- [x] Multi-database architecture (SQLite, Postgres, MySQL).
- [x] Soft deletes.
- [x] Bulk operations (`update_all`, `delete_all`).
- [x] Destructive Guards (New):
  - [x] Prevent `delete_all()` without `.filter()` or `.allow_unsafe()`.

### Phase 7: DevOps (Versioned Migrations) - ✅ COMPLETED
**Mission:** Support team workflows and release readiness. Target: v1.0.0 RC.
- [x] `premix-cli migrate` command family.
- [x] Versioned migration files (`YYYYMMDD_name.sql`).

### Phase 8: The Scale - 📝 PLANNED
**Mission:** High availability and observability. Target: v1.1.0.
- [ ] Read/write splitting (primary + replicas).
- [ ] Connection resolver for multi-tenancy.
- [ ] Metrics collection (New):
  - [x] Pool stats (idle/active) and query latency for Prometheus/Grafana.

### Phase 9: Advanced Relations - ⏳ DEFERRED
**Mission:** Support advanced modeling.
- [ ] Polymorphic relations.

### Phase 10: Legacy Support - 📝 PLANNED
**Mission:** Support brownfield projects.
- [ ] Composite primary keys.
- [ ] Custom Postgres types.

---

## 3. Developer Automation Suite
Standard scripts under `scripts/`: `dev`, `test`, `ci`, `bench`, `release`.

---

## 4. Engineering Risks
- **Compile Time:** Mitigate with lean codegen.
- **Error Messages:** Mitigate with `syn::spanned`.
- **Async Traits:** Mitigate with `BoxFutures`.
- **Accidental Data Loss:** Mitigate with destructive guards.
- **Log Leakage:** Mitigate with sensitive data masking.
