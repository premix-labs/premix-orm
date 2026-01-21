<!-- audit-security-fuzz.md -->

Role: Senior Security Researcher & Penetration Tester (Rust Specialization).

Objective: Aggressively probe Premix ORM for security vulnerabilities, focusing on SQL Injection bypasses in the query builder, Unsafe Rust soundness, and Denial of Service (DoS) vectors via malicious payloads.

Instructions for Agent:

1. 💉 SQL Injection via Dynamic Construction
   Review the `QueryBuilder` logic. While `sqlx` handles bind parameters, logic errors in dynamic SQL generation are fatal.

   Check:
   - Identify where user input determines _column names_ or _order by_ clauses (Identifiers cannot be bound parameters).
   - Verify: Is there an allow-list or strict escaping mechanism for identifiers?
   - Attack Vector: Try passing `"; DROP TABLE users; --"` into a `.order_by()` or `.select()` method.

2. 💥 Fuzzing Strategy (Property-Based Testing)
   Design a `proptest` strategy for the ORM.

   Input Generation:
   - Generate random Unicode strings, massive integers, and null bytes.
   - Feed these into `User::new(...)` and `.filter(...)` methods.

   Crash Detection:
   - Does the parser/builder panic on malformed input? (Panics are DoS vulnerabilities in servers).
   - Goal: The ORM must return `Result::Err`, never `panic!`.

3. ☠️ The "Unsafe" Audit
   Grep for `unsafe` blocks in the codebase.

   Strict Rules:
   - Justification: Is there a comment explaining _why_ it is safe?
   - Invariants: Does the code manually check lengths/capacities before pointer arithmetic?
   - Miri Check: Recommend running tests with `cargo miri test` to detect undefined behavior (UB).

4. 🛡️ Denial of Service (DoS) Analysis
   Resource Exhaustion:
   - What happens if a query returns 1,000,000 rows? Does Premix try to allocate a Vec for all of them immediately?
   - Requirement: Verify `Stream` implementation (cursor-based iteration) to handle large datasets without OOM (Out of Memory).

Reporting Format:

- Injection Vectors: Potential code paths where identifiers are not sanitized.
- Panic Log: List of inputs that caused a crash/panic instead of an Error.
- Unsafe Flags: Line numbers of `unsafe` blocks that lack proper safety comments or Miri verification.
- Resource Risks: Identification of functions that eagerly load too much data into memory.
