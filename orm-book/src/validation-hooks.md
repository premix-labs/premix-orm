# Validation and Hooks

Premix exposes two traits for application-level checks:

- **ModelHooks** for pre/post save behavior.
- **ModelValidation** for structured validation results.

## Current Behavior

By default, both traits provide **no-op behavior** for all derived models.
If you need custom logic, opt in by implementing the traits yourself.

To supply your own hooks or validation, add a `premix` attribute to the model
and implement the trait in your code:

```rust,no_run
use premix_orm::prelude::*;

#[premix(custom_hooks, custom_validation)]
#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

impl premix_orm::ModelHooks for User {
    fn before_save(
        &mut self,
    ) -> impl std::future::Future<Output = Result<(), premix_orm::sqlx::Error>> + Send {
        async move {
            if self.name.trim().is_empty() {
                return Err(premix_orm::sqlx::Error::Protocol("name is empty".into()));
            }
            Ok(())
        }
    }
}

impl premix_orm::ModelValidation for User {
    fn validate(&self) -> Result<(), Vec<premix_orm::ValidationError>> {
        if self.name.trim().is_empty() {
            return Err(vec![premix_orm::ValidationError {
                field: "name".to_string(),
                message: "name is empty".to_string(),
            }]);
        }
        Ok(())
    }
}
```

## Validation (Current)

You can call `validate()` to perform default checks (currently always `Ok(())`):

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let mut user = User { id: 0, name: "Alice".to_string() };
assert!(user.validate().is_ok());
user.save(&pool).await?;
# Ok(())
# }
```

## Hooks (Current)

`save()` calls `before_save` and `after_save`, but these are no-ops today. If
you need logic before saving, do it explicitly in your application code:

```rust,no_run
use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
# Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
let mut user = User { id: 0, name: "Alice".to_string() };

if user.name.trim().is_empty() {
    return Err("name is empty".into());
}

user.save(&pool).await?;
# Ok(())
# }
```

## Planned Improvements

- Make hook and validation errors easier to compose across modules.
