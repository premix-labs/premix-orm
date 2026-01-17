use premix_core::{Model, Premix};
use premix_macros::Model;
use serde_json::json;
use sqlx::SqlitePool;

#[derive(Debug, Model, Clone)]
struct User {
    id: i32,
    name: String,
    age: i32,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Sync table
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;

    println!("--- Creating 10 Users ---");
    for i in 1..=10 {
        let mut user = User {
            id: i,
            name: format!("User {}", i),
            age: i * 10,
            status: "active".to_string(),
        };
        user.save(&pool).await?;
    }

    // Verify initial count
    let count = User::find_in_pool(&pool).all().await?.len();
    println!("Initial count: {}", count);
    assert_eq!(count, 10);

    println!("\n--- Test 1: Bulk Update (Set status='banned' for age > 50) ---");
    // Users 6, 7, 8, 9, 10 have age 60..100
    let start = std::time::Instant::now();
    let updated = User::find_in_pool(&pool)
        .filter("age > 50")
        .update(json!({ "status": "banned" }))
        .await?;
    println!("Updated {} users in {:?}.", updated, start.elapsed());
    assert_eq!(updated, 5);

    let banned_count = User::find_in_pool(&pool)
        .filter("status = 'banned'")
        .all()
        .await?
        .len();
    println!("Users with status 'banned': {}", banned_count);
    assert_eq!(banned_count, 5);

    println!("\n--- Test 2: Bulk Delete (Delete where id <= 3) ---");
    // Users 1, 2, 3
    let start_del = std::time::Instant::now();
    let deleted = User::find_in_pool(&pool).filter("id <= 3").delete().await?;
    println!("Deleted {} users in {:?}.", deleted, start_del.elapsed());
    assert_eq!(deleted, 3);

    let remaining = User::find_in_pool(&pool).all().await?.len();
    println!("Remaining users: {}", remaining);
    assert_eq!(remaining, 7); // 10 - 3 = 7

    println!("\n✅ Bulk Operations Verified Successfully!");
    Ok(())
}
