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

### Phase 3: Relations and Optimization - ✅ COMPLETED
**Mission:** Solve N+1 and provide fluent queries.
- [x] Relation macros (`has_many`, `belongs_to`).
- [x] Eager loading (`.include()`) with O(1) query strategy.

### Phase 4: Developer Experience (DX) - 🔄 UPDATED
**Mission:** Make the ORM usable, testable, and adoptable.
- [x] CLI tool (`premix-cli`).
- [x] Documentation and examples.
- [x] Macro error handling.
- [ ] Test Utilities (New):
  - [ ] Transactional tests (auto-rollback after each test case).
  - [ ] MockDatabase helper.
- [ ] Database Scaffolding (New):
  - [ ] `premix-cli scaffold`: Generate Rust structs from an existing database.
- [ ] Framework Integrations (New):
  - [ ] Official helpers: `premix-axum`, `premix-actix`.

### Phase 5: Enterprise Standard - 🔄 UPDATED
**Mission:** Support real-world complexity, security, and reporting.
- [x] Observability (`tracing`).
- [x] ACID transactions & lifecycle hooks.
- [x] Optimistic locking & validation.
- [ ] Arbitrary Struct Mapping (New):
  - [ ] `Premix::raw("...").fetch_as::<ReportStruct>()`.
- [ ] Sensitive Data Masking (New):
  - [ ] `#[premix(sensitive)]` attribute to redact data in logs.
- [ ] Smart Configuration (New):
  - [ ] Auto-tune pool settings based on environment detection.

### Phase 6: The Versatility - 🔄 UPDATED
**Mission:** Remove limitations and ensure safety.
- [x] Multi-database architecture (SQLite, Postgres, MySQL).
- [x] Soft deletes.
- [x] Bulk operations (`update_all`, `delete_all`).
- [ ] Destructive Guards (New):
  - [ ] Prevent `delete_all()` without `.filter()` or `.allow_unsafe()`.

### Phase 7: DevOps (Versioned Migrations) - ✅ COMPLETED
**Mission:** Support team workflows and release readiness. Target: v1.0.0 RC.
- [x] `premix-cli migrate` command family.
- [x] Versioned migration files (`YYYYMMDD_name.sql`).

### Phase 8: The Scale - 📝 PLANNED
**Mission:** High availability and observability. Target: v1.1.0.
- [ ] Read/write splitting (primary + replicas).
- [ ] Connection resolver for multi-tenancy.
- [ ] Metrics collection (New):
  - [ ] Pool stats (idle/active) and query latency for Prometheus/Grafana.

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
