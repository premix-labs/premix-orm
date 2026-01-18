# Transactions

Premix works with `sqlx` transactions by passing a mutable connection.

```rust
let mut tx = pool.begin().await?;

let mut user = User { id: 0, name: "Alice".to_string() };
user.save(&mut *tx).await?;

let users = User::find_in_tx(&mut *tx).all().await?;

tx.commit().await?;
```

The `IntoExecutor` trait lets `save`, `update`, and query methods accept either
a pool or a transaction connection.
