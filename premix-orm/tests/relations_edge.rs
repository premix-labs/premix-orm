use premix_orm::prelude::*;
use sqlx::Sqlite;

#[derive(Model, Debug, Clone)]
struct RelUser {
    id: i32,
    name: String,
    #[has_many(RelPost)]
    #[premix(ignore)]
    posts: Option<Vec<RelPost>>,
    #[has_many(RelComment)]
    #[premix(ignore)]
    comments: Option<Vec<RelComment>>,
}

#[derive(Model, Debug, Clone)]
#[belongs_to(RelUser)]
struct RelPost {
    id: i32,
    reluser_id: i32,
    title: String,
}

#[derive(Model, Debug, Clone)]
#[belongs_to(RelUser)]
struct RelComment {
    id: i32,
    reluser_id: i32,
    body: String,
}

async fn setup_relation_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, RelUser>(&pool).await.expect("sync");
    Premix::sync::<Sqlite, RelPost>(&pool).await.expect("sync");
    Premix::sync::<Sqlite, RelComment>(&pool)
        .await
        .expect("sync");
    pool
}

#[tokio::test]
async fn sqlite_multi_include_loads_multiple_relations() {
    let pool = setup_relation_pool().await;

    let mut user = RelUser {
        id: 0,
        name: "Rel".to_string(),
        posts: None,
        comments: None,
    };
    user.save(&pool).await.expect("save");

    let mut post = RelPost {
        id: 0,
        reluser_id: user.id,
        title: "Post".to_string(),
    };
    post.save(&pool).await.expect("save");

    let mut comment = RelComment {
        id: 0,
        reluser_id: user.id,
        body: "Comment".to_string(),
    };
    comment.save(&pool).await.expect("save");

    let users = RelUser::find_in_pool(&pool)
        .include(RelUser::posts)
        .include(RelUser::comments)
        .all()
        .await
        .expect("all");
    assert_eq!(users.len(), 1);
    let loaded = &users[0];
    assert_eq!(loaded.posts.as_ref().expect("posts").len(), 1);
    assert_eq!(loaded.comments.as_ref().expect("comments").len(), 1);
}
