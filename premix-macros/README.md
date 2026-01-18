# Premix Macros

Procedural macros for **Premix ORM**.

This crate provides the `#[derive(Model)]` macro, which automatically implements CRUD logic and database mapping for your Rust structs.

## Research Status

This crate is part of a research prototype. APIs may change and production use is not recommended yet.

## Installation

Most users should add `premix-orm` instead. Use `premix-macros` directly only if needed.

```toml
[dependencies]
premix-macros = "1.0.4"
```

## Quick Start

```rust
use premix_macros::Model;
use serde::{Deserialize, Serialize};

#[derive(Model, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}
```

This derives:
- `table_name()`
- `save()` (Create/Insert)
- `find_by_id()`
- `update()`
- `delete()`
- Relationship handling (`has_many`, `belongs_to`)

## License

This project is licensed under the [MIT license](LICENSE).
