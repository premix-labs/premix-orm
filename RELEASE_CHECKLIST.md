## Release Checklist

Use this checklist before publishing a new version.

### Versioning and docs

- [x] Bump versions in `premix-core`, `premix-macros`, `premix-orm`, `premix-cli`.
- [x] Align internal dependency versions (`premix-orm`/`premix-cli` -> `premix-core`/`premix-macros`).
- [x] Update `CHANGELOG.md` with release notes and date.
- [x] Update README and book version strings (`README.md`, `premix-orm/README.md`, `orm-book/`).

### Quality checks

- [x] Format and lint: `scripts/dev/run_fmt.ps1`
- [x] Quick tests: `scripts/test/test_quick.ps1`
- [x] Build workspace: `cargo build`
- [x] Unit/integration tests: `cargo test --workspace --all-features`
- [x] MySQL/Postgres integration tests (Docker + `DATABASE_URL` or `PREMIX_*_URL`)
- [x] Metrics tests: `cargo test --workspace --all-features`

### Publish flow

- [x] Dry run publish (optional): `cargo publish --dry-run` per crate. (premix-core/premix-macros OK; premix-orm/premix-cli blocked until premix-core 1.0.9-alpha is on crates.io)
- [ ] Publish order: `premix-core` -> `premix-macros` -> `premix-orm` -> `premix-cli`
- [ ] Tag release and push tags (optional).

### Post-release

- [ ] Verify crates.io pages update.
- [ ] Announce release if needed.
