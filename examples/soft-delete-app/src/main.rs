use premix_core::{Executor, Model, Premix};
use premix_macros::Model;
use sqlx::SqlitePool;

#[derive(Model, Debug, Default, Clone)]
struct User {
    id: i32,
    name: String,
    deleted_at: Option<String>, // Marker for Soft Delete
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // 1. Sync Schema
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;

    // 2. Create User
    let mut user = User {
        id: 1,
        name: "Alice".to_string(),
        deleted_at: None,
    };
    user.save(&pool).await?;
    println!("Created: {:?}", user);

    // 3. Verify User Exists
    let found = User::find_by_id(&pool, 1).await?;
    assert!(found.is_some(), "User should exist");
    println!("Found: {:?}", found.unwrap());

    // 4. Soft Delete
    println!("Soft Deleting User...");
    user.delete(Executor::Pool(&pool)).await?;
    println!("User soft deleted");

    // 5. Verify User is NOT found by normal find
    let found_after = User::find_by_id(&pool, 1).await?;
    assert!(
        found_after.is_none(),
        "User should NOT be found after soft delete"
    );
    println!("User successfully hidden from find_by_id");

    // 6. Verify with QueryBuilder (default should filter out)
    let found_qb = User::find_in_pool(&pool).filter("id = 1").all().await?;
    assert!(
        found_qb.is_empty(),
        "QueryBuilder should exclude soft deleted records by default"
    );
    println!("User successfully hidden from QueryBuilder");

    // 7. Verify with QueryBuilder + with_deleted()
    let found_with_deleted = User::find_in_pool(&pool)
        .with_deleted()
        .filter("id = 1")
        .all()
        .await?;
    assert!(
        !found_with_deleted.is_empty(),
        "Should find user with .with_deleted()"
    );
    println!("Found (Soft Deleted): {:?}", found_with_deleted[0]);

    // 8. Verify raw SQL (Data Integrity)
    let raw: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = 1")
        .fetch_optional(&pool)
        .await?;
    assert!(raw.is_some());
    let raw_user = raw.unwrap();
    assert!(
        raw_user.deleted_at.is_some(),
        "deleted_at timestamp should be set"
    );
    println!(
        "Raw SQL verification passed: deleted_at = {:?}",
        raw_user.deleted_at
    );

    println!("[OK] All Soft Delete tests passed!");

    Ok(())
}
