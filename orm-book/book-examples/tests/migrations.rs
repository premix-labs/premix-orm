use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[tokio::test]
async fn migrations_auto_sync() -> Result<(), Box<dyn std::error::Error>> {
    let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Ok(())
}
