# premix-actix

Lightweight Actix-web helpers for Premix ORM.

## Research Status

This crate is part of an AI-assisted research prototype. APIs may change and production use is not recommended yet.

## Usage

```rust
use premix_actix::premix_data;
use premix_orm::prelude::*;
use actix_web::{App, HttpServer};

let pool = Premix::smart_sqlite_pool("sqlite:app.db").await?;
let data = premix_data(pool);

HttpServer::new(move || App::new().app_data(data.clone()));
```
