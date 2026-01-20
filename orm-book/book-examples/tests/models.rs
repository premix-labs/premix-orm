use premix_orm::prelude::*;

#[derive(Model)]
struct BasicUser {
    id: i32,
    name: String,
}

#[derive(Model)]
struct IgnoredFieldUser {
    id: i32,
    name: String,

    #[premix(ignore)]
    #[allow(dead_code)]
    in_memory_only: Option<String>,
}

#[tokio::test]
async fn model_helpers_compile_and_generate_sql() -> Result<(), Box<dyn std::error::Error>> {
    let sql = BasicUser::create_table_sql();
    assert!(sql.contains("CREATE TABLE"));

    let pool = Premix::smart_sqlite_pool("sqlite::memory:").await?;
    Premix::sync::<premix_orm::sqlx::Sqlite, IgnoredFieldUser>(&pool).await?;
    Ok(())
}
