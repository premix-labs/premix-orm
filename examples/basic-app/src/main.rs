use premix_core::{Model, Premix};
use premix_macros::Model;
use sqlx::sqlite::SqlitePool;

#[derive(Model)]
struct User {
    id: i32,
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Create Connection Pool (In-memory for speed)
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // 2. Extract SQL from our Macro
    let sql = <User as Model<sqlx::Sqlite>>::create_table_sql();
    println!("Executing: {}", sql);

    // 3. Execute the command on the database
    sqlx::query(&sql).execute(&pool).await?;
    println!("[OK] Table created successfully!");

    // 4. Test Active Record: save()
    let mut user = User {
        id: 1,
        name: "Somchai".to_string(),
        age: 30,
    };

    // Call the generated method!
    user.save(&pool).await?;
    println!("[OK] Row inserted via Active Record!");

    // 5. Test Active Record: find_by_id()
    let found_user = User::find_by_id(&pool, 1).await?;
    if let Some(u) = found_user {
        println!("[OK] Found user: {} (ID: {})", u.name, u.id);
    } else {
        println!("[FAIL] User not found!");
    }

    println!(">> Syncing Database...");
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;
    println!("[OK] Database Synced!");

    Ok(())
}
