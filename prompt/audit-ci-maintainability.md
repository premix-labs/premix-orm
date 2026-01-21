<!-- audit-ci-maintainability.md -->

Role: DevOps Engineer & Open Source Maintainer.

Objective: Establish a robust CI/CD pipeline and enforce code quality standards to ensure Premix ORM remains maintainable and releasable.

Instructions for Agent:

1. 🏗️ CI Pipeline Matrix
   Design the GitHub Actions workflow.

   Matrices needed:
   - OS: Ubuntu-latest, macOS-latest, Windows-latest.
   - Databases: Service containers for Postgres, MySQL, SQLite (verify integration tests against real DBs).
   - Rust Channels: Stable, Beta, Nightly.
   - MSRV (Minimum Supported Rust Version): Test against the oldest version you claim to support.

2. 🧹 Linter & Formatting Standards
   Audit `clippy.toml` and `rustfmt.toml`.
   - Pedantic Clippy: Should we enable `#[warn(clippy::pedantic)]`?
   - Documentation: Enforce `#[deny(missing_docs)]` for all public APIs.
   - Cargo.toml: Check for bloated features. Are dependencies optional (e.g., `features = ["postgres", "runtime-tokio"]`) to keep compile times low?

3. 📦 Release Engineering
   Audit the public API surface (`pub` items).
   - SemVer Check: Use `cargo-semver-checks` to ensure no accidental breaking changes between versions.
   - Feature Flags: specific analysis of additive vs. subtractive features.
   - Crates.io Readiness: Check metadata (license, repository, keywords, categories).

Reporting Format:

- CI Gaps: Missing test environments (e.g., forgot to test on Windows).
- Feature Bloat: Default features that should be optional.
- Dependency Risk: Dependencies that are unmaintained or have known vulnerabilities (cargo-audit).
