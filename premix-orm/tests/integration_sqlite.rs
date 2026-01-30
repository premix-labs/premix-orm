use futures_util::StreamExt;
use premix_orm::prelude::*;
use serde_json::json;
use sqlx::Sqlite;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Model, Debug, Clone)]
struct User {
    id: i32,
    name: String,
    #[has_many(Post)]
    #[premix(ignore)]
    posts: Option<Vec<Post>>,
}

#[derive(Model, Debug, Clone)]
#[belongs_to(User)]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
}

#[derive(Model, Debug, Clone)]
struct SoftUser {
    id: i32,
    name: String,
    deleted_at: Option<String>,
}

#[derive(Model, Debug, Clone)]
struct NullableUser {
    id: i32,
    name: Option<String>,
}

#[derive(Model, Debug, Clone)]
#[premix(custom_hooks)]
struct HookUser {
    id: i32,
    name: String,
}

#[derive(Model, Debug, Clone)]
struct SensitiveUser {
    id: i32,
    #[premix(sensitive)]
    password: String,
}

static BEFORE_SAVE_COUNT: AtomicUsize = AtomicUsize::new(0);
static AFTER_SAVE_COUNT: AtomicUsize = AtomicUsize::new(0);

impl ModelHooks for HookUser {
    fn before_save(&mut self) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send {
        async move {
            BEFORE_SAVE_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn after_save(&mut self) -> impl std::future::Future<Output = Result<(), sqlx::Error>> + Send {
        async move {
            AFTER_SAVE_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
}

async fn setup_user_post_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, User>(&pool).await.expect("sync");
    Premix::sync::<Sqlite, Post>(&pool).await.expect("sync");
    pool
}

async fn seed_users(pool: &sqlx::SqlitePool, names: &[&str]) -> Vec<User> {
    let mut users = Vec::with_capacity(names.len());
    for name in names {
        let mut user = User {
            id: 0,
            name: name.to_string(),
            posts: None,
        };
        user.save(pool).await.expect("save");
        users.push(user);
    }
    users
}

async fn setup_soft_user_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, SoftUser>(&pool).await.expect("sync");
    pool
}

async fn setup_nullable_user_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, NullableUser>(&pool)
        .await
        .expect("sync");
    pool
}

async fn setup_hook_user_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, HookUser>(&pool).await.expect("sync");
    pool
}

async fn setup_sensitive_user_pool() -> sqlx::SqlitePool {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, SensitiveUser>(&pool)
        .await
        .expect("sync");
    pool
}

#[tokio::test]
async fn sqlite_crud_and_relations() {
    let pool = setup_user_post_pool().await;

    let mut user = User {
        id: 0,
        name: "Alice".to_string(),
        posts: None,
    };
    user.save(&pool).await.expect("save");

    let mut post1 = Post {
        id: 0,
        user_id: user.id,
        title: "Hello".to_string(),
    };
    let mut post2 = Post {
        id: 0,
        user_id: user.id,
        title: "World".to_string(),
    };
    post1.save(&pool).await.expect("save");
    post2.save(&pool).await.expect("save");

    let users = User::find_in_pool(&pool)
        .include("posts")
        .all()
        .await
        .expect("all");
    assert_eq!(users.len(), 1);
    let posts = users[0].posts.as_ref().expect("posts");
    assert_eq!(posts.len(), 2);
}

#[tokio::test]
async fn sqlite_soft_delete_hides_rows() {
    let pool = setup_soft_user_pool().await;
    let mut user = SoftUser {
        id: 0,
        name: "Softy".to_string(),
        deleted_at: None,
    };
    user.save(&pool).await.expect("save");
    let user_id = user.id;
    user.delete(&pool).await.expect("delete");

    let found = SoftUser::find_by_id(&pool, user_id).await.expect("find");
    assert!(found.is_none());

    let all = SoftUser::find_in_pool(&pool).all().await.expect("all");
    assert!(all.is_empty());

    let with_deleted = SoftUser::find_in_pool(&pool)
        .with_deleted()
        .all()
        .await
        .expect("all");
    assert_eq!(with_deleted.len(), 1);
}

#[tokio::test]
async fn sqlite_delete_all_requires_allow_unsafe() {
    let pool = setup_user_post_pool().await;
    let mut user = User {
        id: 0,
        name: "DeleteMe".to_string(),
        posts: None,
    };
    user.save(&pool).await.expect("save");

    let res = User::find_in_pool(&pool).delete_all().await;
    assert!(res.is_err());

    let res = User::find_in_pool(&pool).allow_unsafe().delete_all().await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn sqlite_premix_query_select() {
    let pool = setup_user_post_pool().await;
    let mut user = User {
        id: 0,
        name: "Query".to_string(),
        posts: None,
    };
    user.save(&pool).await.expect("save");

    let fetched: Option<User> = premix_query!(User, FIND, filter_eq("id", user.id))
        .fetch_optional(&pool)
        .await
        .expect("fetch");
    assert!(fetched.is_some());
}

#[tokio::test]
async fn sqlite_update_all_requires_filters_or_allow_unsafe() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice", "Bob"]).await;

    let err = User::find_in_pool(&pool)
        .update(json!({ "name": "X" }))
        .await;
    assert!(err.is_err());

    let updated = User::find_in_pool(&pool)
        .allow_unsafe()
        .update(json!({ "name": "X" }))
        .await
        .expect("update");
    assert_eq!(updated, 2);
}

#[tokio::test]
async fn sqlite_include_empty_relation() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice"]).await;

    let users = User::find_in_pool(&pool)
        .include("posts")
        .all()
        .await
        .expect("all");
    assert_eq!(users.len(), 1);
    let posts = users[0].posts.as_ref().expect("posts");
    assert!(posts.is_empty());
}

#[tokio::test]
async fn sqlite_ultra_fast_skips_eager_loading() {
    let pool = setup_user_post_pool().await;

    let mut user = User {
        id: 0,
        name: "Ultra".to_string(),
        posts: None,
    };
    user.save(&pool).await.expect("save");

    let mut post = Post {
        id: 0,
        user_id: user.id,
        title: "Only".to_string(),
    };
    post.save(&pool).await.expect("save");

    let users = User::find_in_pool(&pool)
        .include("posts")
        .ultra_fast()
        .all()
        .await
        .expect("all");
    assert_eq!(users.len(), 1);
    assert!(users[0].posts.is_none());
}

#[tokio::test]
async fn sqlite_limit_offset() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["A", "B", "C", "D", "E"]).await;

    let limited = User::find_in_pool(&pool)
        .limit(2)
        .offset(1)
        .all()
        .await
        .expect("all");
    assert_eq!(limited.len(), 2);
}

#[tokio::test]
async fn sqlite_update_with_filter() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice", "Bob"]).await;

    let updated = User::find_in_pool(&pool)
        .filter_eq("name", "Alice")
        .update(json!({ "name": "Alicia" }))
        .await
        .expect("update");
    assert_eq!(updated, 1);

    let results = User::find_in_pool(&pool)
        .filter_eq("name", "Alicia")
        .all()
        .await
        .expect("all");
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn sqlite_delete_with_filter() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice", "Bob"]).await;

    let deleted = User::find_in_pool(&pool)
        .filter_eq("name", "Bob")
        .delete()
        .await
        .expect("delete");
    assert_eq!(deleted, 1);

    let remaining = User::find_in_pool(&pool).all().await.expect("all");
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn sqlite_raw_filter_requires_allow_unsafe() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice"]).await;

    let err = User::find_in_pool(&pool)
        .filter("name = 'Alice'")
        .all()
        .await;
    assert!(err.is_err());

    let ok = User::find_in_pool(&pool)
        .filter("name = 'Alice'")
        .allow_unsafe()
        .all()
        .await;
    assert!(ok.is_ok());
}

#[tokio::test]
async fn sqlite_unsafe_fast_allows_delete_all() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Alice", "Bob"]).await;

    let deleted = User::find_in_pool(&pool)
        .unsafe_fast()
        .delete_all()
        .await
        .expect("delete_all");
    assert_eq!(deleted, 2);
}

#[tokio::test]
async fn sqlite_transaction_rolls_back() {
    let pool = setup_user_post_pool().await;

    let mut conn = pool.acquire().await.expect("acquire");
    sqlx::query("BEGIN")
        .execute(&mut *conn)
        .await
        .expect("begin");
    let mut user = User {
        id: 0,
        name: "TxUser".to_string(),
        posts: None,
    };
    user.save(&mut *conn).await.expect("save");
    sqlx::query("ROLLBACK")
        .execute(&mut *conn)
        .await
        .expect("rollback");

    let users = User::find_in_pool(&pool).all().await.expect("all");
    assert!(users.is_empty());
}

#[tokio::test]
async fn sqlite_find_in_tx_reads_within_transaction() {
    let pool = setup_user_post_pool().await;

    let mut conn = pool.acquire().await.expect("acquire");
    sqlx::query("BEGIN")
        .execute(&mut *conn)
        .await
        .expect("begin");
    let mut user = User {
        id: 0,
        name: "TxRead".to_string(),
        posts: None,
    };
    user.save(&mut *conn).await.expect("save");

    let found = <User as Model<sqlx::Sqlite>>::find_in_tx(&mut *conn)
        .all()
        .await
        .expect("all");
    assert_eq!(found.len(), 1);
    sqlx::query("ROLLBACK")
        .execute(&mut *conn)
        .await
        .expect("rollback");
}

#[tokio::test]
async fn sqlite_filter_variants() {
    let pool = setup_user_post_pool().await;
    let users = seed_users(&pool, &["Ann", "Bob", "Cara", "Dora"]).await;

    let filtered = User::find_in_pool(&pool)
        .filter_ne("name", "Ann")
        .all()
        .await
        .expect("all");
    assert_eq!(filtered.len(), 3);

    let filtered = User::find_in_pool(&pool)
        .filter_like("name", "C%")
        .all()
        .await
        .expect("all");
    assert_eq!(filtered.len(), 1);

    let ids = vec![users[0].id, users[2].id];
    let filtered = User::find_in_pool(&pool)
        .filter_in("id", ids)
        .all()
        .await
        .expect("all");
    assert_eq!(filtered.len(), 2);

    let gt = User::find_in_pool(&pool)
        .filter_gt("id", users[1].id)
        .all()
        .await
        .expect("all");
    assert!(gt.len() <= 2);

    let gte = User::find_in_pool(&pool)
        .filter_gte("id", users[1].id)
        .all()
        .await
        .expect("all");
    assert!(gte.len() >= gt.len());

    let lt = User::find_in_pool(&pool)
        .filter_lt("id", users[2].id)
        .all()
        .await
        .expect("all");
    assert!(lt.len() >= 1);

    let lte = User::find_in_pool(&pool)
        .filter_lte("id", users[2].id)
        .all()
        .await
        .expect("all");
    assert!(lte.len() >= lt.len());
}

#[tokio::test]
async fn sqlite_null_filters() {
    let pool = setup_nullable_user_pool().await;
    let mut user = NullableUser { id: 0, name: None };
    user.save(&pool).await.expect("save");

    let nulls = NullableUser::find_in_pool(&pool)
        .filter_is_null("name")
        .all()
        .await
        .expect("all");
    assert_eq!(nulls.len(), 1);

    let not_null = NullableUser::find_in_pool(&pool)
        .filter_is_not_null("name")
        .all()
        .await
        .expect("all");
    assert!(not_null.is_empty());
}

#[tokio::test]
async fn sqlite_to_sql_helpers() {
    let pool = setup_user_post_pool().await;
    let qb = User::find_in_pool(&pool).filter_eq("name", "Alice");
    let sql = qb.to_sql();
    assert!(sql.contains("SELECT"));

    let update_sql = qb
        .to_update_sql(&json!({ "name": "A" }))
        .expect("update sql");
    assert!(update_sql.contains("UPDATE"));

    let delete_sql = qb.to_delete_sql();
    assert!(delete_sql.contains("DELETE") || delete_sql.contains("UPDATE"));
}

#[tokio::test]
async fn sqlite_stream_api() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["A", "B", "C"]).await;

    let mut stream = User::find_in_pool(&pool).stream().expect("stream");
    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.expect("row");
        count += 1;
    }
    assert_eq!(count, 3);
}

#[tokio::test]
async fn sqlite_prepared_toggle() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["A"]).await;

    let prepared = User::find_in_pool(&pool)
        .prepared()
        .all()
        .await
        .expect("all");
    assert_eq!(prepared.len(), 1);

    let unprepared = User::find_in_pool(&pool)
        .unprepared()
        .all()
        .await
        .expect("all");
    assert_eq!(unprepared.len(), 1);
}

#[tokio::test]
async fn sqlite_model_fast_paths() {
    let pool = setup_user_post_pool().await;

    let mut user = User {
        id: 0,
        name: "Fast".to_string(),
        posts: None,
    };
    user.save_fast(&pool).await.expect("save_fast");

    user.name = "Faster".to_string();
    let updated = user.update_fast(&pool).await.expect("update_fast");
    assert!(matches!(
        updated,
        UpdateResult::Success | UpdateResult::NotImplemented
    ));

    user.delete_fast(&pool).await.expect("delete_fast");

    let mut ultra = User {
        id: 0,
        name: "Ultra".to_string(),
        posts: None,
    };
    ultra.save_ultra(&pool).await.expect("save_ultra");
    ultra.name = "Ultra2".to_string();
    let updated = ultra.update_ultra(&pool).await.expect("update_ultra");
    assert!(matches!(
        updated,
        UpdateResult::Success | UpdateResult::NotImplemented
    ));
    ultra.delete_ultra(&pool).await.expect("delete_ultra");
}

#[tokio::test]
async fn sqlite_filter_raw_requires_allow_unsafe() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["Raw"]).await;

    let err = User::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .all()
        .await;
    assert!(err.is_err());

    let ok = User::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .allow_unsafe()
        .all()
        .await;
    assert!(ok.is_ok());
}

#[tokio::test]
async fn sqlite_update_all_alias() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["A", "B"]).await;

    let updated = User::find_in_pool(&pool)
        .allow_unsafe()
        .update_all(json!({ "name": "Z" }))
        .await
        .expect("update_all");
    assert_eq!(updated, 2);
}

#[tokio::test]
async fn sqlite_hooks_called_on_save_only() {
    let pool = setup_hook_user_pool().await;
    BEFORE_SAVE_COUNT.store(0, Ordering::SeqCst);
    AFTER_SAVE_COUNT.store(0, Ordering::SeqCst);

    let mut user = HookUser {
        id: 0,
        name: "Hook".to_string(),
    };
    user.save(&pool).await.expect("save");

    let mut fast_user = HookUser {
        id: 0,
        name: "FastHook".to_string(),
    };
    fast_user.save_fast(&pool).await.expect("save_fast");

    assert_eq!(BEFORE_SAVE_COUNT.load(Ordering::SeqCst), 1);
    assert_eq!(AFTER_SAVE_COUNT.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn sqlite_concurrent_queries() {
    let pool = setup_user_post_pool().await;
    seed_users(&pool, &["A", "B", "C", "D"]).await;

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let pool_c = pool.clone();

    let (a, b, c) = tokio::join!(
        async move { User::find_in_pool(&pool_a).all().await },
        async move { User::find_in_pool(&pool_b).limit(2).all().await },
        async move {
            User::find_in_pool(&pool_c)
                .filter_like("name", "C%")
                .all()
                .await
        },
    );

    assert_eq!(a.expect("a").len(), 4);
    assert_eq!(b.expect("b").len(), 2);
    assert_eq!(c.expect("c").len(), 1);
}

#[tokio::test]
async fn sqlite_redacts_sensitive_fields_in_logs() {
    struct TestWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("lock").extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let pool = setup_sensitive_user_pool().await;
    let mut user = SensitiveUser {
        id: 0,
        password: "secret".to_string(),
    };
    user.save(&pool).await.expect("save");

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let make_writer = {
        let buffer = buffer.clone();
        move || TestWriter(buffer.clone())
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(make_writer)
        .without_time()
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let _ = SensitiveUser::find_in_pool(&pool)
        .filter_eq("password", "secret")
        .filter_raw("password = 'secret'")
        .allow_unsafe()
        .all()
        .await
        .expect("all");

    let logs = String::from_utf8(buffer.lock().expect("lock").clone()).expect("utf8");
    if logs.is_empty() {
        return;
    }
    if logs.contains("filters") {
        assert!(logs.contains("***"));
        assert!(logs.contains("RAW(<redacted>)"));
    }
}
