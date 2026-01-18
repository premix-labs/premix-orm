# Premix Core

The foundational library for **Premix ORM**.

`premix-core` provides the essential traits, types, and logic that power the ORM functionality. It handles:
- Database abstraction traits (`SqlDialect`)
- Executor logic (`IntoExecutor`, `Executor`)
- Model traits (`Model`)
- SQLx integration helpers

## Usage

This crate is typically used internally by `premix-orm` or the generated code from `premix-macros`. You generally don't need to depend on it directly unless you are writing custom extensions or low-level database tools.

```toml
[dependencies]
premix-core = "1.0.1"
```

## Features

- **sqlite** (default): Support for SQLite
- **postgres**: Support for PostgreSQL
- **mysql**: Support for MySQL (Partial)

## License

This project is licensed under the [MIT license](LICENSE).
