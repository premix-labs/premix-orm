use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use premix_core::{Executor, Model, Premix};
use premix_macros::Model;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use tracing::info;

// 1. Define your Models
#[derive(Debug, Serialize, Deserialize, Model, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
}

#[tokio::main]
async fn main() {
    // Initialize Tracing
    tracing_subscriber::fmt::init();

    // 3. Connect to Database
    let db_url = "sqlite:premix_axum_demo.db?mode=rwc";
    let pool = SqlitePool::connect(db_url)
        .await
        .expect("Failed to connect to DB");

    // Enable WAL mode for better concurrency
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("Failed to set WAL mode");

    // 4. Run Migrations (or Sync)
    Premix::sync::<Sqlite, User>(&pool)
        .await
        .expect("Failed to sync schema");

    info!("Premix ORM v1.0 schema synced!");

    let state = AppState { db: pool };

    // 5. Setup Router with Full CRUD
    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .with_state(state);

    // 6. Start Server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("Server running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// --- DTOs ---

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UpdateUserDto {
    name: Option<String>,
    email: Option<String>,
}

// --- Handlers ---

// LIST: GET /users
async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    let users = User::find_in_pool(&state.db)
        .all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(users))
}

// CREATE: POST /users
async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let mut user = User {
        id: 0,
        name: payload.name,
        email: payload.email,
    };

    user.save(Executor::Pool(&state.db)).await.map_err(|e| {
        info!("Failed to save user: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Created user: {:?}", user);

    Ok((StatusCode::CREATED, Json(user)))
}

// READ: GET /users/{id}
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, StatusCode> {
    match User::find_by_id(Executor::Pool(&state.db), id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// UPDATE: PUT /users/{id}
async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<User>, StatusCode> {
    // Find existing user
    let mut user = match User::find_by_id(Executor::Pool(&state.db), id).await {
        Ok(Some(u)) => u,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Apply updates
    if let Some(name) = payload.name {
        user.name = name;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }

    // Save changes
    match user.update(Executor::Pool(&state.db)).await {
        Ok(premix_core::UpdateResult::Success) => Ok(Json(user)),
        Ok(premix_core::UpdateResult::NotFound) => Err(StatusCode::NOT_FOUND),
        Ok(premix_core::UpdateResult::VersionConflict) => Err(StatusCode::CONFLICT),
        Ok(premix_core::UpdateResult::NotImplemented) => Err(StatusCode::NOT_IMPLEMENTED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// DELETE: DELETE /users/{id}
async fn delete_user(State(state): State<AppState>, Path(id): Path<i32>) -> StatusCode {
    // Find existing user
    let mut user = match User::find_by_id(Executor::Pool(&state.db), id).await {
        Ok(Some(u)) => u,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Delete
    match user.delete(Executor::Pool(&state.db)).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
