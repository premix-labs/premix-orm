# Master Flowplan: Premix ORM

## 🚀 Project Concept
**Concept:** The "Holy Grail" of Rust ORMs.
**Slogan:** "Write Rust, run optimized SQL."

## 🎯 1. Core Philosophy (5 Pillars of a Great ORM)

### 1. ⚡ The "Zero-Overhead" Abstraction (Fastest)
> *"Your code should be as fast as handwriting raw SQL."*
- **Premix Approach:** Uses **Rust Macros** to "write code for humans" at compile time. At runtime, it's 100% pre-prepared Raw SQL.
- **Goal:** Runtime Benchmark should be equal to raw `sqlx` (0% Overhead).

### 2. 🧠 The "Mental Model Match" (Easiest)
> *"Code should look like the way you think, not the way database stores."*
- **Premix Approach:** Connects the Object world (`User`) with the Table world (`users`) as seamlessly as possible.
    - **Naming Convention:** `User` -> `users` automatically.
    - **Active Record:** `user.save()`, `User::find(1)`.
    - **Auto Migration:** Change Struct -> Database updates accordingly.

### 3. 🛡️ The "Impossible-to-Fail" Safety (Most Stable)
> *"If it compiles, it runs without SQL errors."*
- **Premix Approach:** Moves all errors to Compile Time.
    - Type a field name wrong -> **Compile Error**.
    - Type mismatch -> **Compile Error**.
- **Goal:** Eliminate all "Runtime Surprises" (crashes during execution).

### 4. 🔍 The "Glass Box" Transparency (Transparent)
> *"Magic is good, but Black Magic is bad."*
- **Premix Approach:** Features "Show SQL" or `to_sql()` so Developers can see what the Macro generates.
- **Goal:** Developers must feel they "control" the SQL.

### 5. 🚪 The "Graceful Escape Hatch" (Flexible)
> *"Easy things should be easy, hard things should be possible."*
- **Premix Approach:** Allows mixing Raw SQL with ORM smoothly for complex queries.
- **Example:** `User::raw_sql("SELECT * FROM ...").fetch_all()`

---

## 🗺️ 2. Development Flowplan

### 🏗️ Phase 0: Setup & Architecture (The Beginning) - ✅ COMPLETED
**Mission:** Lay the foundation for the project structure to be scalable.

- [x] **Project Initialization:** Create Cargo Workspace `premix-orm`.
- [x] **Module Separation:** Separate `premix-core` (Runtime) and `premix-macros` (Compiler).
- [x] **Basic Macro:** `#[derive(Model)]` can compile successfully.
- [x] **Table Name Generation:** Convert Struct name `User` -> `users` automatically.
- [x] **Column Extraction:** Read Fields (`id`, `name`) into strings.

**Tech Stack:**
- **Database Driver:** `sqlx` (Standing on the shoulders of giants for Connection Pool & Async).
- **Macro Engine:** `syn`, `quote`, `proc-macro2`.
- **Runtime:** `tokio`.

### 🛠️ Phase 1: The "CRUD" Engine (Short-term Goal) - ✅ COMPLETED
**Mission:** Make basic saving and reading of data work.

- [x] **Type Mapping System:** Create Logic in Macro to convert `i32` -> `INTEGER`, `String` -> `TEXT`.
- [x] **Connection Integration:** Connect `premix-core` to `sqlx::PgPool`.

✅ **Milestone 1:** Can call `User::new().save().await` to save to a real Database.

### 🔮 Phase 2: The "Migration" Magic (Key Selling Point) - ✅ COMPLETED
**Mission:** Allow Users not to touch even a single line of SQL.

- [x] **Schema Introspection:** Write code to read all Rust Structs in the project.
- [x] **Diff Engine:** Compare Structs with the real Database; notify if they don't match.
- [x] **Sync Command:** Create `Premix::migrate()` function that runs `CREATE TABLE` or `ALTER TABLE` automatically.

✅ **Milestone 2:** Add a Field in a Rust Struct, run it, and the Database updates immediately.

### 🧠 Phase 3: The "Relations" & Optimization (Maximum Speed) - ✅ COMPLETED
**Mission:** Solve N+1 Query problems and Table Joining.

- [x] **Relation Macros (The Glue):**
	- [x] `#[has_many(Post)]` : Parent knows it has children.
    - [x] `#[belongs_to(User)]` : Child knows its parent.
- [x] **Query Builder (The Fluent API):**
    - [x] `Model::find()` entry point.
    - [x] `.filter()`, `.limit()`, `.offset()` support.
- [x] **Eager Loading (The Speed):**
    - [x] `.include("posts")` support.
    - [x] In-Application Join strategy (batch WHERE IN queries).
    - [x] `#[premix(ignore)]` for non-column fields.
- [x] **Performance Tuning:** Analyze Overhead and reduce N+1 to O(1) Database Queries.

✅ **Milestone 3:** Fetch 100 Users with their Posts in a single Query (No N+1).

### 🛠️ Phase 4: Developer Experience (Real-world Usage) - ✅ COMPLETED
**Mission:** Make it truly usable for developers.

- [x] **CLI Tool:** Create `premix-cli` for terminal commands.
- [x] **Documentation:** Write complete documentation (The Book of Premix) and Example Projects.
- [x] **Error Handling:** Use `syn::Error` to provide Macro-level warnings (Spanned Errors).

✅ **Milestone 4:** Premix is a complete Ecosystem, both Engine and DX.

### 🏢 Phase 5: Enterprise Standard (Perfection) - ✅ COMPLETED
**Mission:** Support real-world complexity.

- [x] **Observability:** Support `tracing` instrumentation to see which queries are slow.
- [x] **Transactions:** ACID system with Manual Pattern (`pool.begin()`, `save_with`, `commit`).
- [x] **Lifecycle Hooks:** `before_save`, `after_save` for hashing passwords or sending emails.
- [x] **Concurrency Control:** `update()` method with infrastructure for Optimistic Locking.
- [x] **Data Validation:** `validate()` method with infrastructure for attribute-based validation.

> [!NOTE]
> **New API in Phase 5:**
> - `model.save_with(&mut *tx)` - Save inside transaction
> - `Model::find_with(&mut *tx)` - Query inside transaction
> - `model.update(&pool)` - Update existing record
> - `model.validate()` - Validate before save

✅ **Milestone 5:** Premix is ready for use in Banking or large-scale Enterprise systems.

### 🌍 Phase 6: The "Versatility" - ✅ COMPLETED
**Mission:** Remove all limitations, support every need.

- [x] **Multi-Database Architecture:**
    - [x] Created `SqlDialect` trait (`placeholder`, `primary_key_type`, `map_type`).
    - [x] Made `Model<DB>`, `QueryBuilder<DB>` generic over database type.
    - [x] Macro generates concrete `impl Model<Sqlite>` (fully generic caused type inference issues).
    - [x] PostgreSQL Support (`sqlx::Postgres`) - via `#[cfg(feature)]`.
    - [x] MySQL Support (`sqlx::Mysql`) - via `#[cfg(feature)]`.
- [x] **Soft Deletes:**
    - [x] `#[derive(SoftDelete)]` macro (Impl in `derive(Model)`).
    - [x] `.delete()` sets `deleted_at` instead of removing row.
    - [x] Default queries filter out soft-deleted rows.
- [x] **Bulk Operations (The Performance Beasts):**
    - [x] `User::update_all(json!({ "status": "active" })).filter("age > 18")`.
    - [x] `User::delete_all().filter("status = 'banned'")`.
- [x] **Advanced Types:**
    - [x] JSON/JSONB support (via `serde_json`).
    - [ ] Enum mapping (Partial support via string).

✅ **Milestone 6:** Premix supports Multi-DB Architecture. Concrete Postgres/MySQL to be added in next versions.

### 🚚 Phase 7: The "DevOps" (Versioned Migrations) - ✅ COMPLETED
**Mission:** Move from "Solo Dev" tools to "Team" tools.
**Target:** Premix ORM v1.0.0 (Release)

- [x] **Versioned Migrations:**
    - [x] `premix-cli migrate` command family.
    - [x] `create`: Generate `YYYYMMDDHHMMSS_name.sql`.
    - [x] `up`: Apply pending migrations using `Migrator`.
    - [x] `Migrator`: Core logic to track versions in `_premix_migrations` table.

✅ **Milestone 7:** Full Migration System completed. Premix ORM is ready for v1.0.0 Release.

### ⚖️ Phase 8: The "Scale" (High Availability)
**Mission:** Support apps with millions of users.
**Target:** Premix ORM v1.2.0 (Scale)

- [ ] **Read/Write Splitting:**
    - [ ] Automatic routing of `SELECT` to Replicas and `INSERT/UPDATE` to Primary.
    - [ ] Configuration: `DATABASE_URL` (Primary) + `DATABASE_READ_URLS` (Replicas).
- [ ] **Connection Resolver:** Dynamic database switching (Multi-Tenancy).

### 🎭 Phase 9: Advanced Relations (Optional Future)
**Mission:** Support complex, niche data modeling.
**Status:** Deferred (Complexity vs Value Trade-off)

- [ ] **Polymorphic Relations:**
    - [ ] `#[polymorphic]` attribute.
    - [ ] Support `imageable_type` and `imageable_id` logic.
- [ ] **Declarative Migrations:** Schema definition file approach.

### 🏛️ Phase 10: The "Legacy" (Compatibility)
**Mission:** Support brownfield projects.

- [ ] **Composite Primary Keys:** `#[primary_key(id, type)]`.
- [ ] **Custom Types:** Full support for Postgres Enums and Custom Domains.

### �🚀 Optional: The "Futurism" (Preparing for the next 10-20 years)
**Mission:** Adapt to Mega-Trends of the future (AI, Edge, Decentralization).

- [ ] **AI-Native & Vectors:** Support `Vector` Type and Semantic Search (`pgvector`) to be the backbone for AI Apps.
- [ ] **Edge & Wasm Ready:** Support running on Browser or Edge Servers (Compile targets: `wasm32-unknown-unknown`).
- [ ] **Local-First Sync (CRDTs):** Support offline-first data syncing (no constant REST API dependency).
- [ ] **Adaptive Self-Optimization:** A system that learns Query patterns and suggests Indexes or auto-tunes itself (Smart AI DBA).

✅ **Milestone (Optional):** Premix is a "Data Layer" of the future, not just a simple ORM.

---

## 🤖 3. Developer Automation Suite

We use a comprehensive suite of PowerShell scripts to standardize workflows.
Located in `scripts/`, organized by category:

### 📂 `scripts/dev` (Daily Development)
- **`run_fmt.ps1`**: Format code (`cargo fmt`) and fix clippy warnings (`cargo clippy --fix`).
- **`run_clean.ps1`**: Deep clean of build artifacts and database files.
- **`gen_docs.ps1`**: Generate rustdoc and mdBook documentation.

### � `scripts/test` (Verification)
- **`test_quick.ps1`**: Fast "Smoke Test" (Build + Run Basic App).
- **`test_examples.ps1`**: Run all example apps to ensure no regressions.
- **`test_migration.ps1`**: E2E test for the Migration CLI and SQL application.

### 📂 `scripts/ci` (Quality Assurance)
- **`check_all.ps1`**: Full workspace check (Build, Test, Clippy) used in CI pipelines.
- **`check_audit.ps1`**: Security vulnerability scan (`cargo audit`).
- **`check_coverage.ps1`**: Code coverage report (`cargo tarpaulin`).

### 📂 `scripts/bench` (Performance)
- **`bench_orm.ps1`**: Compare Premix overhead against Raw SQLx.
- **`bench_io.ps1`**: Heavy I/O benchmark (Partial/Postgres).

### 📂 `scripts/release` (Deployment)
- **`run_publish.ps1`**: Automated crates.io publication with safety checks.

---

## �🖼️ 4. Architecture Diagram (Mental Model)

### Working Flow
1.  **Compile Time (Left):**
    `User writes Code` -> `Premix Macros process` -> `Generates Optimized Rust Code with embedded SQL`.
2.  **Runtime (Right):**
    `App starts` -> `Calls generated Code` -> `Sends via sqlx` -> `Database` (extremely fast because no need to rethink SQL).

---

## ⚠️ 5. Engineering Risks
To create a world-class ORM, we must overcome these challenges:

### 1. 🐌 Compile Time Explosion
- **Risk:** Heavily processing Macros can noticeably slow down compilation.
- **Solution:** Must optimize Code Generation to be as small and fast as possible (Lean) and support Incremental Compilation.

### 2. 🧩 Error Message from Hell
- **Risk:** When Macros fail, Errors often point to the wrong place or are unintelligible.
- **Solution:** Use `syn::spanned` to target Errors accurately to the original Struct or Field name.

### 3. 🕸️ Async in Traits
- **Risk:** Rust still has limitations with Async Traits (e.g., lifetimes and Send/Sync).
- **Solution:** Carefully design Ownership and use BoxFutures where necessary.
