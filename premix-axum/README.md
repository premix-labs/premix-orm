# premix-axum

Lightweight Axum helpers for Premix ORM.

## Usage

```rust
use premix_axum::PremixState;
use premix_orm::prelude::*;
use axum::{extract::State, Router};

let pool = Premix::smart_sqlite_pool("sqlite:app.db").await?;
let state = PremixState::new(pool);

let app = Router::new().with_state(state);
```
