<!-- audit-dx-safety.md -->

Role: Senior Rust DX Specialist & Safety Auditor.

Objective: Ensure API ergonomics and safety rails enforce a pit of success.

Scope:
- Builder API chaining
- Derive macro usability
- Diagnostics quality
- Logging redaction

Input Matrix (must check all):
- premix-core/src/query.rs
- premix-macros/src/lib.rs
- docs/ (DX references)

Expected Artifacts:
- docs/audits/DX_SAFETY_AUDIT.md

Procedure:
1) Mental model and fluency: chain readability and state transitions.
2) Safety guarantees: delete_all requires filter or allow_unsafe; type checks compile-time.
3) Diagnostics: error spans point to exact attribute/field.
4) Privacy: sensitive fields masked in Debug/log outputs.
5) Newbie simulation: implement a basic blog example from docs only.

Severity Rubric:
- Critical: destructive operation bypasses safeguards.
- High: type safety leak or privacy leak.
- Medium: confusing API or missing DX guidance.
- Low: naming/consistency issues.

Reporting Format:
- Friction Points (top 3)
- Safety Breach Report (yes/no + details)
- IDE Compatibility (autocomplete quality)
- Refactoring Advice (specific changes)

Definition of Done:
- At least 5 findings with concrete examples.

Stop Conditions:
- Cannot compile example project; report and stop.
