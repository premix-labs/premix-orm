## Release Checklist

Use this checklist before publishing a new version.

### Versioning and docs

- [ ] Bump versions in `premix-core`, `premix-macros`, `premix-orm`, `premix-cli`.
- [ ] Align internal dependency versions (`premix-orm`/`premix-cli` -> `premix-core`/`premix-macros`).
- [ ] Update `CHANGELOG.md` with release notes and date.
- [ ] Update README and book version strings (`README.md`, `premix-orm/README.md`, `orm-book/`).

### Quality checks

- [ ] Format and lint: `scripts/dev/run_fmt.ps1`
- [ ] Quick tests: `scripts/test/test_quick.ps1`
- [ ] Build workspace: `cargo build`

### Publish flow

- [ ] Dry run publish (optional): `cargo publish --dry-run` per crate. (premix-core/premix-macros OK; premix-orm/premix-cli blocked until premix-core 1.0.7-alpha is on crates.io)
- [ ] Publish order: `premix-core` -> `premix-macros` -> `premix-orm` -> `premix-cli`
- [ ] Tag release and push tags (optional).

### Post-release

- [ ] Verify crates.io pages update.
- [ ] Announce release if needed.
