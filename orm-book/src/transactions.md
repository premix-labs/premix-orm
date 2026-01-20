# Transactions

Premix works with `sqlx` transactions by accepting either a pool or a mutable
connection. This allows `save`, `update`, and query builder calls to work
inside a transaction.

## Basic Transaction Flow

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
let mut tx = pool.begin().await?;

let mut user = User { id: 0, name: "Alice".to_string() };
user.save(&mut *tx).await?;

let users = User::find_in_tx(&mut *tx).all().await?;

tx.commit().await?;
# Ok(())
# }
```

## Why `&mut *tx`?

`sqlx::Transaction` dereferences to a connection. Premix accepts a mutable
connection so both `&pool` and `&mut tx` can be passed to the same API via
`IntoExecutor`.

## Rollback

If any step fails, call `tx.rollback().await?` or let the transaction drop
without committing. The database will revert the changes.

## Recommended Usage

- Wrap multi-step writes in a transaction.
- Keep transactions short to avoid locks.
- Use transactions with optimistic locking when data races are possible.
