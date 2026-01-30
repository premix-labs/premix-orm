# Contributing to Premix ORM

Thank you for your interest in contributing to Premix ORM, the "Holy Grail" of Rust ORMs! We welcome contributions from everyone.

## 🛠️ Development Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/premix-orm.git
    cd premix-orm
    ```

2.  **Install Prerequisites:**
    - Rust (latest stable)
    - PowerShell (for running helper scripts)

3.  **Verify the setup:**
    Run the quick test script to ensure everything compiles:
    ```powershell
    ./scripts/test/test_quick.ps1
    ```

## 📂 Project Structure

Understanding the workspace layout helps you navigate the code:

```text
premix-orm/
├── premix-orm/        # 📦 The Facade: Unified entry point (Crates.io default)
├── premix-core/       # 🧠 The Runtime: Connection pooling, SQL builder, traits
├── premix-macros/     # ⚙️ The Compiler: proc-macros for #[derive(Model)]
├── premix-cli/        # 🛠️ The Tool: CLI for syncing and migrations
├── benchmarks/        # 📊 The Proof: Comparison with SeaORM/Rbatis/SQLx
├── examples/          # 💡 The Demos: Real-world usage examples
└── scripts/           # 🐚 The Helpers: PowerShell scripts organized by category
```

## 🧪 Running Tests

We have a suite of PowerShell scripts to make testing easy:

- **Run all checks (CI):** `./scripts/ci/check_all.ps1`
- **Run Unit Tests:** `cargo test`
- **Run Benchmarks:** `./scripts/bench/bench_orm.ps1`

## 📝 Coding Standards

- **Formatting:** We use `rustfmt`. Run `./scripts/dev/run_fmt.ps1` before committing.
- **Linting:** We use `clippy`. The `run_fmt.ps1` script also runs clippy.
- **Commits:** Please use conventional commits (e.g., `feat: add new macro`, `fix: resolve N+1 issue`).

## 🤝 Pull Request Process

1.  Fork the repo and create your branch from `main`.
2.  Add tests for any new features or bug fixes.
3.  Ensure all tests pass and your code is formatted.
4.  Open a Pull Request with a clear description of your changes.

## 🐛 Reporting Bugs

Please inspect the `docs/guide/TESTING_GUIDE.md` for information on how to reproduce issues. Open an issue on GitHub with a minimal reproduction case.

Happy Coding! 🚀
