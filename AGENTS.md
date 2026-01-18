## AGENTS

This file defines how AI agents should work in this repository.

Scope
- Default to changes inside `premix-orm/` unless instructed otherwise.
- Do not edit generated files in `target/` or `coverage/`.

Repo map
- Crates: `premix-core/`, `premix-orm/`, `premix-macros/`, `premix-cli/`
- Docs: `README.md`, `docs/`, `orm-book/`
- Dev scripts: `scripts/dev/`
- Examples: `examples/`

Workflow
- Prefer small, focused diffs.
- Update docs when behavior changes.
- Keep ASCII unless the file already uses Unicode.

Commands
- Setup: `scripts/dev/setup_env.ps1`
- Format: `scripts/dev/run_fmt.ps1`
- Docs: `scripts/dev/gen_docs.ps1`

Testing
- If tests are needed, document what you ran and why.
- Prefer targeted tests over full suites unless requested.

Issues and support
- Follow `SECURITY.md` for security topics.
- Use `CONTRIBUTING.md` for contribution rules.
