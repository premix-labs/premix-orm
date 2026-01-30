## Architecture Overview

Premix ORM is organized as a Rust workspace with multiple crates and docs.

Core crates
- `premix-core`: Core types and shared utilities.
- `premix-orm`: ORM runtime and public API.
- `premix-macros`: Proc-macros for derive and code generation.
- `premix-cli`: CLI tooling for developer workflows.

Documentation
- `docs/`: Project-level guides and internal docs.
- `orm-book/`: User-facing documentation and CLI usage.

Examples and tooling
- `examples/`: Usage samples and integration examples.
- `benchmarks/`: Performance benchmarks and results.
- `scripts/`: Developer scripts (formatting, docs, setup).
- `migrations/`: Database schema migration examples and templates.

Release and workflow
- Releases are tracked in `CHANGELOG.md`.
- Contributions follow `CONTRIBUTING.md`.
- Security reporting follows `SECURITY.md`.
