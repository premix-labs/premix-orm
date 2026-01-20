Role: Senior Rust Compiler Engineer & Systems Architect.

Objective: Conduct a deep-dive forensic audit of the Premix ORM source code and its expanded macro artifacts. Ensure "Zero-Overhead" promises are mathematically true and Macro Hygiene is bulletproof.

Instructions for Agent:

1. 🧬 Macro Hygiene & Namespace Safety (Critical)
Fully Qualified Paths Check:

Scan the generated code (via cargo expand). Does it use Option or ::std::option::Option?

Rule: All types used in generated code MUST use absolute paths (e.g., ::sqlx::..., ::std::...) to prevent breakage if a user shadows standard types.

Variable Shadowing:

Check generated variable names inside functions. Are they prefixed (e.g., __premix_internal_var) to avoid colliding with user fields?

Verify use of Span::mixed_site() vs Span::call_site() for proper identifier hygiene.

2. ⚡ Zero-Overhead "White-Box" Analysis
Compile-Time vs. Runtime SQL:

Analyze the to_sql() implementation. Is the SQL string constructed using format! at runtime (Heap allocation)?

Optimization Goal: It should be constructed using concat! or const generics at compile-time where possible.

The "Clone" Hunt:

Look for .clone() or .to_owned() calls inside the generated impl Model.

Challenge: Can these be replaced with Cow<'a, str> or reference borrowing?

Trait Dispatch:

Verify that generated code relies on Static Dispatch (Generics/Monomorphization) and explicitly avoids Box<dyn Trait> in hot paths.

3. 🧵 Async/Concurrency Safety
Send + Sync Verification:

The generated Futures (from async methods) MUST be Send to work with Tokio/Axum.

Check if any Rc<RefCell<...>> (which is not Thread Safe) slipped into the generated code. It should be Arc<Mutex<...>> or Arc<RwLock<...>> if shared state is needed.

4. 📍 Diagnostic Quality (Span Analysis)
Error Attribution:

Review the proc_macro source (in premix-macros).

Test: If the macro panics or returns a compile_error!, does it use quote_spanned! pointing to the specific field/struct causing the issue?

Fail: If the error points to the #[derive(Model)] line generally, it fails the audit.

Reporting Format:

Hygiene Violations: List any types/functions missing fully qualified paths (e.g., using Result instead of ::std::result::Result).

Allocation Hotspots: Line numbers in the expanded code where implicit heap allocations occur.

Concurrency Risks: Types or Futures that fail the Send + Sync check.

Refactoring Roadmap: Concrete steps to move runtime logic to compile-time (const-eval).