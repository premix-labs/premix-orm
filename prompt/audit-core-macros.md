<!-- audit-core-macros.md -->

Role: Senior Rust Compiler Engineer & Systems Architect.

Objective: Audit procedural macro output for hygiene, zero-overhead behavior, and diagnostic quality.

Scope:
- premix-macros derive and query macros
- Expanded code for Model and query builder

Prerequisites / Tools:
- cargo expand

Input Matrix (must check all):
- premix-macros/src/lib.rs
- premix-macros/src/relations.rs
- premix-macros/src/static_query.rs

Expected Artifacts:
- docs/audits/ZERO_OVERHEAD_AUDIT.md (macro section)

Run Order:
1) cargo expand -p premix-macros
2) Review expanded code for Model and relations

Procedure:
1) Macro hygiene: fully qualified paths and prefixed identifiers.
2) Zero-overhead: locate clone/to_owned and dynamic dispatch in generated code.
3) Async safety: confirm generated futures are Send where required.
4) Diagnostics: verify spans point to the exact offending field/attribute.

Severity Rubric:
- Critical: hygiene break causing user code failure.
- High: allocations or dyn dispatch in hot path.
- Medium: non-ideal spans or minor allocs.
- Low: cosmetic inconsistencies.

Reporting Format:
- Hygiene Violations (type/path, location)
- Allocation Hotspots (line references)
- Concurrency Risks (Send/Sync issues)
- Refactoring Roadmap (compile-time vs runtime)

Definition of Done:
- At least one expanded artifact reviewed per macro.
- Hygiene and diagnostic checks completed.

Stop Conditions:
- cargo expand not available or expansion fails; report and stop.
