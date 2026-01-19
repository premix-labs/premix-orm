use premix_orm::prelude::*;

#[derive(Model, Debug)]
struct User {
    id: i32,
    name: String,
}

#[tokio::test]
async fn getting_started_flow() -> Result<(), Box<dyn std::error::Error>> {
    let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let mut user = User { id: 0, name: "Alice".to_string() };
    user.save(&pool).await?;

    let users = User::find_in_pool(&pool)
        .filter("name = 'Alice'")
        .all()
        .await?;

    assert_eq!(users.len(), 1);
    Ok(())
}
