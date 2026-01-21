# premix-metrics

Prometheus metrics helpers for Premix ORM.

## Research Status

This crate is part of an AI-assisted research prototype. APIs may change and production use is not recommended yet.

## Usage

```rust
use premix_metrics::{install_prometheus_recorder, record_pool_stats};
use premix_orm::prelude::*;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let handle = install_prometheus_recorder()?;
let pool = Premix::smart_sqlite_pool("sqlite:app.db").await?;

record_pool_stats(&pool, "sqlite");
println!("{}", handle.render());
# Ok(())
# }
```

Enable query latency metrics by compiling `premix-orm` with the `metrics` feature:

```toml
premix-orm = { version = "1.0.6-alpha", features = ["metrics"] }
```
