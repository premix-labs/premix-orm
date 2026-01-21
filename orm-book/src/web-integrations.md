# Web Integrations

Premix provides built-in helpers for popular Rust web frameworks through feature flags.

## Axum Integration

Enable the `axum` feature in `Cargo.toml`:

```toml
premix-orm = { version = "1.0.7-alpha", features = ["axum"] }
```

### PremixState

Use `PremixState` to wrap your database pool and share it across handlers:

```rust,no_run
use ax_app::{Router, routing::get, extract::State};
use premix_orm::prelude::*;
use premix_orm::integrations::axum::PremixState;
use std::sync::Arc;

struct AppState {
    db: PremixState<sqlx::Sqlite>,
}

async fn handler(State(state): State<Arc<AppState>>) -> String {
    // Access the pool directly or via standard extraction
    format!("Pool size: {}", state.db.pool.size())
}

#[tokio::main]
async fn main() {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await.unwrap();
    let state = Arc::new(AppState {
        db: PremixState::new(pool),
    });

    let app = Router::new()
        .route("/", get(handler))
        .with_state(state);

    // ... serve app
}
```

## Actix-web Integration

Enable the `actix` feature in `Cargo.toml`:

```toml
premix-orm = { version = "1.0.7-alpha", features = ["actix"] }
```

### PremixData

Use `PremixData` as a type alias for `actix_web::web::Data<sqlx::Pool<DB>>`:

```rust,no_run
use actix_web::{web, App, HttpServer, Responder};
use premix_orm::prelude::*;

async fn index(db: premix_orm::integrations::actix::PremixData<sqlx::Sqlite>) -> impl Responder {
    format!("Pool size: {}", db.size())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await.unwrap();
    let data = premix_orm::integrations::actix::premix_data(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## Metrics & Observability

Enable the `metrics` feature in `premix-core` or `premix-orm`:

```toml
premix-orm = { version = "1.0.7-alpha", features = ["metrics"] }
```

### Prometheus Support

Premix can export pool statistics to Prometheus:

```rust,no_run
use premix_orm::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Install the global recorder
    let handle = premix_orm::metrics::install_prometheus_recorder()?;

    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

    // 2. Record stats periodically
    loop {
        premix_orm::metrics::record_pool_stats(&pool, "main_db");
        // ... sleep
        # break;
    }

    // 3. Render handle for scraping endpoint
    Ok(())
}
```
