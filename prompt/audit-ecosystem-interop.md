<!-- audit-ecosystem-interop.md -->

Role: Rust Ecosystem Integrator & Library Maintainer.

Objective: Validate seamless integration with common Rust ecosystem crates and runtimes.

Scope:
- Serde integration
- Tracing/logging
- UUID/time/JSON types
- Runtime compatibility

Input Matrix (must check all):
- premix-core/src/model.rs
- premix-core/src/query.rs
- docs/ (interop sections)

Expected Artifacts:
- docs/audits/ECOSYSTEM_INTEROP_AUDIT.md

Procedure:
1) Serde compatibility: serialize/deserialize; attribute conflicts.
2) Observability: tracing spans and redaction behavior.
3) Common types: UUID, chrono/time, serde_json::Value mappings.
4) Runtime neutrality: verify no hard tokio-only behavior unless documented.

Severity Rubric:
- Critical: interop blocked for common types or runtime.
- High: missing tracing or privacy leak.
- Medium: manual conversion burden.
- Low: minor doc gaps.

Reporting Format:
- Friction Points (manual conversions)
- Logging Gaps (missing spans)
- Serde Conflicts (attribute collisions)

Definition of Done:
- At least 3 interop scenarios validated.

Stop Conditions:
- Required features not enabled; report and stop.
