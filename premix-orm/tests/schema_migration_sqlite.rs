use premix_orm::prelude::*;
use premix_orm::schema::{SchemaColumn, diff_sqlite_schema, sqlite_migration_sql};
use sqlx::Sqlite;

#[derive(Model, Debug, Clone)]
struct SchemaUser {
    id: i32,
    name: String,
}

#[tokio::test]
async fn sqlite_schema_diff_empty_when_synced() {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, SchemaUser>(&pool)
        .await
        .expect("sync");

    let expected = vec![SchemaUser::schema()];
    let diff = diff_sqlite_schema(&pool, &expected).await.expect("diff");
    assert!(diff.is_empty());
}

#[tokio::test]
async fn sqlite_schema_diff_detects_missing_column() {
    let pool = Premix::smart_sqlite_pool("sqlite::memory:")
        .await
        .expect("pool");
    Premix::sync::<Sqlite, SchemaUser>(&pool)
        .await
        .expect("sync");

    let mut altered = SchemaUser::schema();
    altered.columns.push(SchemaColumn {
        name: "extra".to_string(),
        sql_type: "TEXT".to_string(),
        nullable: true,
        primary_key: false,
    });
    let expected = vec![altered];
    let diff = diff_sqlite_schema(&pool, &expected).await.expect("diff");
    assert!(!diff.is_empty());

    let sql = sqlite_migration_sql(&expected, &diff);
    assert!(!sql.is_empty());
}
