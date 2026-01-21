use premix_orm::prelude::*;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Create Connection Pool (In-memory for speed)
    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;

    // 2. Auto-Sync Schema (Creates tables automatically)
    println!(">> Syncing Schema...");
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;
    println!("[OK] Table created via Auto-Sync!");

    // 3. Test Active Record: save()
    let mut user = User {
        id: 1,
        name: "Somchai".to_string(),
        age: 30,
    };

    // Call the generated method!
    user.save(&pool).await?;
    println!("[OK] Row inserted via Active Record!");

    // 4. Test Active Record: find_by_id()
    let found_user = User::find_by_id(&pool, 1).await?;
    if let Some(u) = found_user {
        println!("[OK] Found user: {} (ID: {})", u.name, u.id);
    } else {
        println!("[FAIL] User not found!");
    }

    Ok(())
}
