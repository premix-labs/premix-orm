# Onboarding Audit

Date: 2026-01-31
Scope: README.md, orm-book, examples/

## Friction Log (Time-to-Hello-World)
1) Followed examples/basic-app (first runnable example). `cargo run` succeeded and printed expected output. (examples/basic-app)
2) README Quick Start compiles, but does not explicitly call out save/update semantics or new CLI flags.

## Broken Snippets
- orm-book/src/cli-usage.md: "premix init" described as placeholder, but CLI now scaffolds templates. Doc is out of date.
- orm-book/src/cli-usage.md: schema example imports `premix_orm::schema_models`, but this macro is not re-exported in premix-orm; should use `premix_core::schema::schema_models` or re-export. (orm-book/src/cli-usage.md)
- orm-book/src/cli-usage.md: schema example uses `premix_orm::schema` and `schema_models` without showing required feature flags; may fail on default features.

## Content Gaps
- CLI flags `--dry-run`/`--yes` and .env auto-load are not documented in orm-book CLI usage.
- Save/update semantics (save updates when id != 0) are not called out in Getting Started or CLI docs.
- No explicit "how to run examples" guidance in README (path + cargo run commands).

## Magic Warnings
- CLI schema diff/sync now scans `src/` directly; README should mention the helper binaries are no longer required.
- Migration down on SQLite can be destructive; warning is missing from orm-book CLI usage.

Definition of Done: 5+ concrete findings listed.
