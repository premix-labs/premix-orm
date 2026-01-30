#[cfg(feature = "sqlite")]
use premix_core::migrator::{Migration, Migrator};
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;

#[cfg(feature = "sqlite")]
async fn setup_pool() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.expect("pool")
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn sqlite_migrator_run_and_rollback() {
    let pool = setup_pool().await;
    let migrator = Migrator::new(pool.clone());

    let migrations = vec![Migration {
        version: "0001_create_items".to_string(),
        name: "create_items".to_string(),
        up_sql: "CREATE TABLE items (id INTEGER PRIMARY KEY);".to_string(),
        down_sql: "DROP TABLE items;".to_string(),
    }];

    migrator.run(migrations.clone()).await.expect("run");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 1);

    let rolled = migrator.rollback_last(migrations).await.expect("rollback");
    assert!(rolled);
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count_after, 0);
}
