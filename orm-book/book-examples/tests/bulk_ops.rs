use premix_orm::prelude::*;
use serde_json::json;

#[derive(Model)]
struct User {
    id: i32,
    age: i32,
    status: String,
    deleted_at: Option<String>,
}

#[tokio::test]
async fn bulk_ops_examples() -> Result<(), Box<dyn std::error::Error>> {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;

    let mut user = User {
        id: 0,
        age: 21,
        status: "inactive".to_string(),
        deleted_at: None,
    };
    user.save(&pool).await?;

    let updated = User::find_in_pool(&pool)
        .filter_gt("age", 18)
        .update(json!({ "status": "active" }))
        .await?;
    assert_eq!(updated, 1);

    let removed = User::find_in_pool(&pool)
        .filter_eq("status", "active")
        .delete()
        .await?;
    assert_eq!(removed, 1);

    let all = User::find_in_pool(&pool)
        .with_deleted()
        .all()
        .await?;
    assert_eq!(all.len(), 1);

    Ok(())
}
