use premix_orm::prelude::*;

#[derive(Model, Clone, Debug, PartialEq)]
struct User {
    id: i32,
    name: String,
}

#[derive(Model, Debug)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,

    #[belongs_to(User)]
    #[premix(ignore)]
    user: Option<User>,
}

#[tokio::test]
async fn eager_load_belongs_to_populates_parent() -> Result<(), Box<dyn std::error::Error>> {
    let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;

    let mut user = User {
        id: 0,
        name: "Alice".to_string(),
    };
    user.save(&pool).await?;

    let mut post = Post {
        id: 0,
        user_id: user.id,
        title: "Hello".to_string(),
        user: None,
    };
    post.save(&pool).await?;

    let posts = Post::find_in_pool(&pool).include("user").all().await?;
    assert_eq!(posts.len(), 1);
    let loaded = posts[0].user.clone().expect("user loaded");
    assert_eq!(loaded, user);
    Ok(())
}
