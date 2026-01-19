use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
    deleted_at: Option<String>,
}

#[tokio::test]
async fn query_builder_examples() -> Result<(), Box<dyn std::error::Error>> {
    let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let mut user = User { id: 0, age: 21, deleted_at: None };
    user.save(&pool).await?;

    let found = User::find_by_id(&pool, user.id).await?;
    assert!(found.is_some());

    let users = User::find_in_pool(&pool)
        .filter("age > 18")
        .limit(20)
        .offset(0)
        .all()
        .await?;
    assert_eq!(users.len(), 1);

    let rows = User::raw_sql("SELECT * FROM users WHERE age > 18")
        .fetch_all(&pool)
        .await?;
    assert_eq!(rows.len(), 1);

    let users_with_deleted = User::find_in_pool(&pool)
        .with_deleted()
        .limit(100)
        .all()
        .await?;
    assert_eq!(users_with_deleted.len(), 1);

    Ok(())
}
