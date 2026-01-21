use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use premix_core::{Model, Premix};
use premix_macros::Model;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

// 1. Define Model
#[derive(Model, Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    role: String,
}

// 2. Shared State
struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A. Setup Database
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

    // B. Auto Migration (The Magic)
    println!(">> Starting Premix Auto-Migration...");
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;
    println!("[OK] Database Synced!");

    // C. Setup Axum Router
    let shared_state = Arc::new(AppState { pool });
    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .with_state(shared_state);

    // D. Run Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(">> Server running at http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

// Handler 1: Create User
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<User>,
) -> Result<Json<User>, StatusCode> {
    println!(">> Saving user: {:?}", payload);

    // Use Premix Save!
    match payload.save(&state.pool).await {
        Ok(_) => Ok(Json(payload)),
        Err(e) => {
            eprintln!("Error saving user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handler 2: Get User
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<User>, StatusCode> {
    // Use Premix Find!
    match User::find_by_id(&state.pool, id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
