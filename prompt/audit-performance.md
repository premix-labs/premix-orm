<!-- audit-performance.md -->

Role: Senior Rust Performance Engineer & Systems Architect.

Objective: Rigorously validate the "Zero-Overhead" claim of Premix ORM. Ensure that the abstraction layer introduces no measurable latency compared to raw sqlx and optimizes for modern CPU architectures (L1/L2 cache, branch prediction).

Instructions for Agent:

1. 🛠️ Tooling & Instrumentation
   Ensure cargo-bloat and cargo-expand are available.

If possible, simulate a profile-guided analysis by looking for std::alloc calls in the hot path.

2. 🧠 Deep Performance Audit
   Analyze the source and expanded code for:

Heap vs Stack (Zero-Allocation Path):

Check if the Query Builder uses String concatenation or Vec resizing inside loops.

Optimization Goal: Encourage the use of fmt::Write into a pre-allocated buffer or StackString concepts. Identify any Box<dyn ...> or Arc that could be replaced with Static Dispatch or stack-based traits.

Monomorphization & Code Bloat:

Use cargo bloat to see if Generic implementations for every Model lead to an explosion in binary size.

Verification: Ensure that shared logic is moved to non-generic functions (thin wrappers) where appropriate to keep the Instruction Cache (I-Cache) clean.

Cache Locality (O(1) Eager Loading):

Examine the data structures used in .include(). Are you using HashMap with high collision potential?

Optimization Goal: Evaluate if a "Flat Map" or sorted Vec (binary search) would be more cache-friendly for small relation sets. Ensure data is stored contiguously to minimize L1/L2 cache misses.

Branch Prediction & Assembly Logic:

Compare the generated code of a Premix query against a manually written sqlx query.

Check: Are there extra match arms or if let checks that the compiler cannot optimize away? Look for "Indirect Calls" (vtable lookups) that break the CPU's branch predictor.

3. ⚖️ The "Golden Ratio" Comparison
   Perform a logic-to-logic comparison:

Premix: User::find().filter(id.eq(1)).fetch_one()

Raw: sqlx::query_as!(User, "SELECT \* FROM users WHERE id = ?", 1)

Count the number of transformations between the user's call and the database driver's execution. Every extra move/copy is a penalty.

Reporting Format:

Allocation Hotspots: List specific lines where heap allocation occurs in the query path.

Dispatch Analysis: Identify where Dynamic Dispatch is used and propose a Static Dispatch alternative.

Cache Efficiency Score: Assessment of the Eager Loading data layout.

Instruction Overhead: Estimated "extra" CPU instructions added by the ORM compared to raw SQL.
