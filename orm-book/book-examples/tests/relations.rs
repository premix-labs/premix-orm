use premix_orm::prelude::*;

#[derive(Model)]
#[has_many(Post)]
struct User {
    id: i32,
    name: String,

    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

#[tokio::test]
async fn relations_examples() -> Result<(), Box<dyn std::error::Error>> {
    let pool = premix_orm::sqlx::SqlitePool::connect("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, User>(&pool).await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, Post>(&pool).await?;

    let mut user = User { id: 0, name: "Alice".to_string(), posts: None };
    user.save(&pool).await?;

    let mut post = Post { id: 0, user_id: user.id, title: "Hello".to_string() };
    post.save(&pool).await?;

    let users = User::find_in_pool(&pool)
        .include("posts")
        .all()
        .await?;
    assert_eq!(users.len(), 1);

    let posts = users[0].posts_lazy(&pool).await?;
    assert_eq!(posts.len(), 1);

    Ok(())
}
