use premix_core::Model;
use premix_macros::Model;
use sqlx::SqlitePool;

// 1. Parent Model
#[derive(Model)]
#[has_many(Post)]
struct User {
    id: i32,
    name: String,
}

// 2. Child Model
#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32, // Foreign Key
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // A. Sync Tables
    sqlx::query(&<User as Model<sqlx::Sqlite>>::create_table_sql())
        .execute(&pool)
        .await?;
    sqlx::query(&<Post as Model<sqlx::Sqlite>>::create_table_sql())
        .execute(&pool)
        .await?;

    // B. Seed Data
    let mut user = User {
        id: 1,
        name: "Alice".to_string(),
    };
    user.save(&pool).await?;

    let mut post1 = Post {
        id: 1,
        user_id: 1,
        title: "Hello Rust".to_string(),
    };
    let mut post2 = Post {
        id: 2,
        user_id: 1,
        title: "Macro Magic".to_string(),
    };
    post1.save(&pool).await?;
    post2.save(&pool).await?;

    // C. Test Relations - Relation methods take just the executor (DB is inferred from SQLite pool)
    println!("--- Testing User.posts() ---");
    let posts = user.posts_lazy(&pool).await?;
    for p in &posts {
        println!("- Found Post: {}", p.title);
    }
    assert_eq!(posts.len(), 2);

    println!("\n--- Testing Post.user() ---");
    let parent = post1.user(&pool).await?;
    if let Some(u) = parent {
        println!("- Post belongs to: {}", u.name);
        assert_eq!(u.name, "Alice");
    } else {
        println!("- Orphan post?");
    }

    println!("\n✅ Relation Test Passed!");
    Ok(())
}
