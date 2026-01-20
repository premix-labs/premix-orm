use premix_orm::prelude::*;
use premix_orm::ModelValidation;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[tokio::test]
async fn validation_and_hooks_flow() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let mut user = User { id: 0, name: "Alice".to_string() };
    assert!(user.validate().is_ok());
    user.save(&pool).await?;
    Ok(())
}
