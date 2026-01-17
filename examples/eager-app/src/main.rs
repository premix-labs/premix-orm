use premix_core::{Model, Premix};
use premix_macros::Model;
use sqlx::SqlitePool;

// --- Models ---
#[derive(Model, Debug)]
struct User {
    id: i32,
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model, Debug)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // 1. Sync Tables
    Premix::sync::<sqlx::Sqlite, User>(&pool).await?;
    Premix::sync::<sqlx::Sqlite, Post>(&pool).await?;

    // 2. Seed Data
    let mut user1 = User {
        id: 1,
        name: "Alice".to_string(),
        posts: None,
    };
    user1.save(&pool).await?;

    let mut post1 = Post {
        id: 101,
        user_id: 1,
        title: "Alice's First Post".to_string(),
    };
    post1.save(&pool).await?;
    let mut post2 = Post {
        id: 102,
        user_id: 1,
        title: "Alice's Second Post".to_string(),
    };
    post2.save(&pool).await?;

    // 3. Eager Load Test
    println!("--- Testing Eager Loading ---");
    let users = User::find_in_pool(&pool)
        .include("posts") // Should trigger eager_load for "posts"
        .all()
        .await?;

    let loaded_user = &users[0];
    println!("User: {:?}", loaded_user);

    if let Some(posts) = &loaded_user.posts {
        println!("Loaded {} posts!", posts.len());
        for p in posts {
            println!("- {}", p.title);
        }
        assert_eq!(posts.len(), 2);
    } else {
        panic!("Posts were not loaded! Eager loading failed.");
    }

    Ok(())
}
