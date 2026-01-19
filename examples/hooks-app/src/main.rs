use premix_core::{Model, ModelHooks, Premix};
use premix_macros::Model;
use sqlx::sqlite::SqlitePool;

#[derive(Model, Debug)]
#[premix(custom_hooks)]
struct User {
    id: i32,
    name: String,
    role: String,
}

// Override Hooks!
#[premix_core::async_trait::async_trait]
impl ModelHooks for User {
    async fn before_save(&mut self) -> Result<(), sqlx::Error> {
        println!("🎣 [before_save] Hook triggered for: {}", self.name);
        if self.role == "admin" {
            self.name = format!("⭐ {}", self.name);
            println!("   -> Modified name to: {}", self.name);
        }
        Ok(())
    }

    async fn after_save(&mut self) -> Result<(), sqlx::Error> {
        println!("🎣 [after_save] User saved successfully!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Setup
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;

    println!("--- Test 1: Normal User ---");
    let mut user1 = User {
        id: 1,
        name: "Alice".to_string(),
        role: "user".to_string(),
    };
    user1.save(&pool).await?;

    // Verify DB content
    let saved_user1 = User::find_by_id(&pool, 1).await?.unwrap();
    println!("DB Result: {}", saved_user1.name);
    assert_eq!(saved_user1.name, "Alice");

    println!("\n--- Test 2: Admin User (Hook should modify name) ---");
    let mut user2 = User {
        id: 2,
        name: "Bob".to_string(),
        role: "admin".to_string(),
    };
    user2.save(&pool).await?;

    // Verify DB content
    let saved_user2 = User::find_by_id(&pool, 2).await?.unwrap();
    println!("DB Result: {}", saved_user2.name);
    assert_eq!(saved_user2.name, "⭐ Bob");

    Ok(())
}
