# Premix Macros

Procedural macros for **Premix ORM**.

This crate provides the `#[derive(Model)]` macro, which automatically implements the CRUD logic and database mapping for your Rust structs.

## Usage

Add this to your `Cargo.toml` (usually via `premix-orm` or `premix-macros` directly if needed):

```toml
[dependencies]
premix-macros = "1.0.1"
```

## Example

```rust
use premix_macros::Model;

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
