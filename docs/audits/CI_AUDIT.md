# CI Audit

Date: 2026-01-31
Scope: .github/workflows, Cargo.toml, .clippy.toml, .rustfmt.toml

## CI Gaps
- Services matrix is defined for all OSes, but GitHub Actions services only run on Linux; Postgres/MySQL integration coverage likely only on ubuntu-latest (no explicit conditional). (.github/workflows/ci.yml)
- No security/dependency audit step (cargo-audit/cargo-deny) in CI. (.github/workflows/ci.yml)
- No semver compatibility checks (cargo-semver-checks) in CI. (.github/workflows/ci.yml)
- No docs build or rustdoc warnings gate (cargo doc -D warnings). (.github/workflows/ci.yml)

## Feature Bloat
- CI runs `cargo test --workspace --all-features` on every OS/channel; this can pull in heavy DB features and inflate matrix time without targeted coverage. (.github/workflows/ci.yml)

## Dependency Risk
- clippy MSRV is set to 1.75.0 while CI MSRV check uses 1.85.0, which can hide MSRV regressions for clippy-specific lints. (.clippy.toml, .github/workflows/ci.yml)

## Release Blockers
- Rustfmt is configured with edition 2021 while workspace uses 2024; formatting may differ across editions. (.rustfmt.toml, Cargo.toml)
- Crate metadata lacks documentation/homepage fields in several crates (premix-cli, premix-core), which is a release polish gap.

Definition of Done: 4 CI gaps identified.
