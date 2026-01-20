use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
}

#[tokio::test]
async fn transactions_flow() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let mut tx = pool.begin().await?;

    let mut user = User { id: 0, name: "Alice".to_string() };
    user.save(&mut *tx).await?;

    let users = User::find_in_tx(&mut *tx).all().await?;
    assert_eq!(users.len(), 1);

    tx.commit().await?;
    Ok(())
}
