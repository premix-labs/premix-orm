use premix_core::{
    Model as PremixModel, Premix, UpdateResult,
    sqlx::{self, Sqlite, SqlitePool},
};
use premix_macros::Model;

#[derive(Model, Debug)]
#[has_many(Post)]
struct User {
    id: i32,
    name: String,
    deleted_at: Option<String>,
    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model, Debug)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

#[derive(Model, Debug)]
struct Account {
    id: i32,
    version: i32,
    name: String,
}

#[test]
fn derive_model_generates_metadata() {
    assert_eq!(<User as PremixModel<Sqlite>>::table_name(), "users");
    assert!(
        <User as PremixModel<Sqlite>>::create_table_sql()
            .contains("CREATE TABLE IF NOT EXISTS users")
    );
    assert!(<User as PremixModel<Sqlite>>::list_columns().contains(&"name".to_string()));
    assert!(<User as PremixModel<Sqlite>>::list_columns().contains(&"deleted_at".to_string()));
    assert!(<User as PremixModel<Sqlite>>::has_soft_delete());
    assert!(!<Post as PremixModel<Sqlite>>::has_soft_delete());
}

#[tokio::test]
async fn derive_model_relations_work() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, User>(&pool).await.unwrap();
    Premix::sync::<Sqlite, Post>(&pool).await.unwrap();

    sqlx::query("INSERT INTO users (id, name, deleted_at) VALUES (1, 'Alice', NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id, name, deleted_at) VALUES (2, 'Bob', NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO posts (id, user_id, title) VALUES (1, 1, 'Post A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO posts (id, user_id, title) VALUES (2, 2, 'Post B')")
        .execute(&pool)
        .await
        .unwrap();
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        deleted_at: None,
        posts: None,
    };
    let posts = user.posts_lazy::<_, Sqlite>(&pool).await.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].user_id, 1);

    let post = Post {
        id: 1,
        user_id: 1,
        title: "Hello".to_string(),
    };
    let parent = post.user::<_, Sqlite>(&pool).await.unwrap();
    let parent = parent.unwrap();
    assert_eq!(parent.id, 1);

    let mut users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            deleted_at: None,
            posts: None,
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            deleted_at: None,
            posts: None,
        },
    ];
    <User as PremixModel<Sqlite>>::eager_load(
        &mut users,
        "posts",
        premix_core::Executor::Pool(&pool),
    )
    .await
    .unwrap();
    assert_eq!(users[0].posts.as_ref().unwrap().len(), 1);
    assert_eq!(users[1].posts.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn derive_model_update_handles_optimistic_locking() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, Account>(&pool).await.unwrap();

    sqlx::query("INSERT INTO accounts (id, name, version) VALUES (1, 'Alice', 1)")
        .execute(&pool)
        .await
        .unwrap();

    let mut account = Account {
        id: 1,
        name: "Alice".to_string(),
        version: 1,
    };
    let res = account.update(&pool).await.unwrap();
    assert_eq!(res, UpdateResult::Success);
    assert_eq!(account.version, 2);

    let mut stale = Account {
        id: 1,
        name: "Stale".to_string(),
        version: 1,
    };
    let res = stale.update(&pool).await.unwrap();
    assert_eq!(res, UpdateResult::VersionConflict);
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(version, 2);
}

#[tokio::test]
async fn derive_model_delete_respects_soft_delete() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, User>(&pool).await.unwrap();

    sqlx::query("INSERT INTO users (id, name, deleted_at) VALUES (1, 'Alice', NULL)")
        .execute(&pool)
        .await
        .unwrap();

    let mut user = User {
        id: 1,
        name: "Alice".to_string(),
        deleted_at: None,
        posts: None,
    };
    user.delete(&pool).await.unwrap();
    assert_eq!(user.deleted_at.as_deref(), Some("DELETED"));

    let deleted_at: Option<String> =
        sqlx::query_scalar("SELECT deleted_at FROM users WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(deleted_at.is_some());
    let remaining = <User as PremixModel<Sqlite>>::find_in_pool(&pool)
        .all()
        .await
        .unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn derive_model_delete_removes_row_for_hard_delete() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, Post>(&pool).await.unwrap();

    sqlx::query("INSERT INTO posts (id, user_id, title) VALUES (1, 1, 'Hello')")
        .execute(&pool)
        .await
        .unwrap();

    let mut post = Post {
        id: 1,
        user_id: 1,
        title: "Hello".to_string(),
    };
    post.delete(&pool).await.unwrap();

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn derive_model_save_sets_id_and_honors_explicit_id() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, User>(&pool).await.unwrap();

    let mut new_user = User {
        id: 0,
        name: "New".to_string(),
        deleted_at: None,
        posts: None,
    };
    new_user.save(&pool).await.unwrap();
    assert!(new_user.id > 0);

    let mut explicit = User {
        id: 42,
        name: "Explicit".to_string(),
        deleted_at: None,
        posts: None,
    };
    explicit.save(&pool).await.unwrap();
    assert_eq!(explicit.id, 42);
}

#[tokio::test]
async fn derive_model_find_by_id_respects_soft_delete() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, User>(&pool).await.unwrap();

    sqlx::query("INSERT INTO users (id, name, deleted_at) VALUES (1, 'Alive', NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id, name, deleted_at) VALUES (2, 'Gone', 'x')")
        .execute(&pool)
        .await
        .unwrap();

    let alive = <User as PremixModel<Sqlite>>::find_by_id(&pool, 1)
        .await
        .unwrap();
    assert!(alive.is_some());

    let deleted = <User as PremixModel<Sqlite>>::find_by_id(&pool, 2)
        .await
        .unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn derive_model_update_without_version_returns_not_found() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    Premix::sync::<Sqlite, Post>(&pool).await.unwrap();

    let mut missing = Post {
        id: 999,
        user_id: 1,
        title: "Missing".to_string(),
    };
    let res = missing.update(&pool).await.unwrap();
    assert_eq!(res, UpdateResult::NotFound);

    sqlx::query("INSERT INTO posts (id, user_id, title) VALUES (1, 1, 'Old')")
        .execute(&pool)
        .await
        .unwrap();
    let mut existing = Post {
        id: 1,
        user_id: 1,
        title: "New".to_string(),
    };
    let res = existing.update(&pool).await.unwrap();
    assert_eq!(res, UpdateResult::Success);
}
