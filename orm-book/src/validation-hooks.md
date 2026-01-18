# Validation and Hooks

Premix exposes two traits for application-level logic:

## Hooks

Implement `ModelHooks` to run code before or after saving:

```rust
use premix_orm::ModelHooks;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[premix_orm::async_trait::async_trait]
impl ModelHooks for User {
    async fn before_save(&mut self) -> Result<(), premix_orm::sqlx::Error> {
        if self.name.trim().is_empty() {
            return Err(premix_orm::sqlx::Error::Protocol("name is empty".into()));
        }
        Ok(())
    }
}
```

`save()` automatically calls `before_save` and `after_save`.

## Validation

Implement `ModelValidation` when you want to validate data explicitly:

```rust
use premix_orm::{ModelValidation, ValidationError};

impl ModelValidation for User {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        if self.name.trim().is_empty() {
            return Err(vec![ValidationError {
                field: "name".to_string(),
                message: "name is required".to_string(),
            }]);
        }
        Ok(())
    }
}
```

Validation is not automatically invoked; call `validate()` in your own logic
before saving.
