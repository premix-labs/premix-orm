<!-- audit-security-fuzz.md -->

Role: Senior Security Researcher & Penetration Tester (Rust).

Objective: Identify SQL injection, unsafe Rust soundness risks, and DoS vectors.

Scope:
- Query builder identifier handling
- Unsafe blocks
- Large result streaming

Input Matrix (must check all):
- premix-core/src/query.rs
- premix-core/src/executor.rs
- premix-macros/src (SQL generation paths)

Expected Artifacts:
- docs/audits/SECURITY_AUDIT.md

Procedure:
1) Injection review: identifier construction and escaping.
2) Fuzz strategy: proptest inputs for filters/values.
3) Unsafe audit: enumerate unsafe blocks and safety comments.
4) DoS analysis: verify streaming for large results.

Severity Rubric:
- Critical: injection vector or UB.
- High: panic on malformed input.
- Medium: missing fuzz coverage.
- Low: minor safety doc gaps.

Reporting Format:
- Injection Vectors
- Panic Log
- Unsafe Flags
- Resource Risks

Definition of Done:
- At least 5 findings or explicit "none found" with evidence.

Stop Conditions:
- Missing fuzz tools; report and stop.
