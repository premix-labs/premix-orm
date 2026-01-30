# CORE MACROS AUDIT

Status: Macro expansion executed (cargo expand -p premix-macros --lib).

Hygiene Violations:
- Expanded output shows unqualified Result/Vec usage. Prefer ::std::result::Result and ::std::vec::Vec in generated code to avoid shadowing.

Allocation Hotspots:
- Expanded relations code uses format! to build SQL strings at runtime. This allocates per call.

Concurrency Risks:
- No explicit Send/Sync violations observed in expansion pass.

Refactoring Roadmap:
1) Use fully qualified std paths in generated code.
2) Move SQL building to compile-time or cache per model/operation.
3) Use prefix for generated identifiers consistently.

Notes:
- Expansion output is large; this audit focused on hygiene and allocation hints.
