<!-- audit-ci-maintainability.md -->

Role: DevOps Engineer & Open Source Maintainer.

Objective: Ensure CI coverage, quality gates, and release readiness are robust and reproducible.

Scope:
- GitHub Actions matrix
- Lint/format policy
- SemVer and crate metadata

Input Matrix (must check all):
- .github/workflows/ (CI)
- Cargo.toml (workspace + crates)
- clippy.toml, rustfmt.toml

Expected Artifacts:
- docs/audits/CI_AUDIT.md

Procedure:
1) CI matrix: OS, Rust channels, MSRV, DB services.
2) Quality gates: rustfmt, clippy, doc tests.
3) Dependency hygiene: optional features and cargo-audit.
4) Release readiness: cargo-semver-checks and metadata completeness.

Severity Rubric:
- Critical: missing CI coverage for supported platforms.
- High: missing quality gates or semver risk.
- Medium: feature bloat or metadata gaps.
- Low: minor config polish.

Reporting Format:
- CI Gaps
- Feature Bloat
- Dependency Risk
- Release Blockers

Definition of Done:
- At least 3 CI gaps or explicit "none found" with evidence.

Stop Conditions:
- CI config missing; report and stop.
