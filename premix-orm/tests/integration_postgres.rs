#![cfg(feature = "postgres")]

use futures_util::StreamExt;
use premix_orm::prelude::*;
use serde_json::json;
use sqlx::PgPool;

#[derive(Model, Debug, Clone)]
struct PgUser {
    id: i32,
    name: String,
}

#[tokio::test]
async fn postgres_crud_smoke() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();

    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    let mut user = PgUser {
        id: 0,
        name: "PgUser".to_string(),
    };
    user.save(&pool).await.expect("save");

    let found = PgUser::find_by_id(&pool, user.id).await.expect("find");
    assert!(found.is_some());

    let updated = PgUser::find_in_pool(&pool)
        .filter_eq("name", "PgUser")
        .update(json!({ "name": "PgUser2" }))
        .await
        .expect("update");
    assert_eq!(updated, 1);

    let deleted = PgUser::find_in_pool(&pool)
        .filter_eq("name", "PgUser2")
        .delete()
        .await
        .expect("delete");
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn postgres_filters_limit_offset_prepared() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    for name in ["A", "B", "C", "D"] {
        let mut user = PgUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let filtered = PgUser::find_in_pool(&pool)
        .filter_ne("name", "A")
        .limit(2)
        .offset(1)
        .prepared()
        .all()
        .await
        .expect("all");
    assert_eq!(filtered.len(), 2);

    let unprepared = PgUser::find_in_pool(&pool)
        .filter_gt("id", 0)
        .unprepared()
        .all()
        .await
        .expect("all");
    assert_eq!(unprepared.len(), 4);
}

#[tokio::test]
async fn postgres_raw_filter_requires_allow_unsafe() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    let mut user = PgUser {
        id: 0,
        name: "Raw".to_string(),
    };
    user.save(&pool).await.expect("save");

    let err = PgUser::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .all()
        .await;
    assert!(err.is_err());

    let ok = PgUser::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .allow_unsafe()
        .all()
        .await;
    assert!(ok.is_ok());
}

#[tokio::test]
async fn postgres_to_sql_uses_numbered_placeholders() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let qb = PgUser::find_in_pool(&pool).filter_eq("name", "A");
    let sql = qb.to_sql();
    assert!(sql.contains("$1"));
    let update_sql = qb.to_update_sql(&json!({ "name": "B" })).expect("update");
    assert!(update_sql.contains("$"));
}

#[tokio::test]
async fn postgres_stream_api() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    for name in ["A", "B"] {
        let mut user = PgUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let mut stream = PgUser::find_in_pool(&pool).stream().expect("stream");
    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.expect("row");
        count += 1;
    }
    assert_eq!(count, 2);
}

#[tokio::test]
async fn postgres_filter_null_like_in() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    for name in ["Ann", "Bob", "Cara"] {
        let mut user = PgUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let like = PgUser::find_in_pool(&pool)
        .filter_like("name", "C%")
        .all()
        .await
        .expect("all");
    assert_eq!(like.len(), 1);

    let ids = vec![like[0].id];
    let in_filter = PgUser::find_in_pool(&pool)
        .filter_in("id", ids)
        .all()
        .await
        .expect("all");
    assert_eq!(in_filter.len(), 1);
}

#[tokio::test]
async fn postgres_transaction_rolls_back() {
    let pool = match get_pg_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <PgUser as Model<sqlx::Postgres>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS \"{}\"", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::Postgres, PgUser>(&pool)
        .await
        .expect("sync");

    let mut conn = pool.acquire().await.expect("acquire");
    let conn = &mut *conn;
    sqlx::query("BEGIN")
        .execute(&mut *conn)
        .await
        .expect("begin");
    let mut user = PgUser {
        id: 0,
        name: "Tx".to_string(),
    };
    user.save(&mut *conn).await.expect("save");
    sqlx::query("ROLLBACK")
        .execute(&mut *conn)
        .await
        .expect("rollback");

    let found = PgUser::find_in_pool(&pool).all().await.expect("all");
    assert!(found.is_empty());
}

async fn get_pg_pool() -> Option<PgPool> {
    let db_url = std::env::var("DATABASE_URL").ok()?;
    if !db_url.starts_with("postgres://") && !db_url.starts_with("postgresql://") {
        return None;
    }
    Some(sqlx::PgPool::connect(&db_url).await.ok()?)
}
