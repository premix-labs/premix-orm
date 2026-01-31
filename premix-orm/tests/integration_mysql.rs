#![cfg(feature = "mysql")]

use futures_util::StreamExt;
use premix_orm::prelude::*;
use serde_json::json;
use sqlx::MySqlPool;

#[derive(Model, Debug, Clone)]
struct MyUser {
    id: i32,
    name: String,
}

#[tokio::test]
async fn mysql_crud_smoke() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <MyUser as Model<sqlx::MySql>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS `{}`", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();

    Premix::sync::<sqlx::MySql, MyUser>(&pool)
        .await
        .expect("sync");

    let mut user = MyUser {
        id: 0,
        name: "MyUser".to_string(),
    };
    user.save(&pool).await.expect("save");

    let found = MyUser::find_by_id(&pool, user.id).await.expect("find");
    assert!(found.is_some());

    let updated = MyUser::find_in_pool(&pool)
        .filter_eq("name", "MyUser")
        .update(json!({ "name": "MyUser2" }))
        .await
        .expect("update");
    assert_eq!(updated, 1);

    let deleted = MyUser::find_in_pool(&pool)
        .filter_eq("name", "MyUser2")
        .delete()
        .await
        .expect("delete");
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn mysql_filters_limit_offset_prepared() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <MyUser as Model<sqlx::MySql>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS `{}`", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::MySql, MyUser>(&pool)
        .await
        .expect("sync");

    for name in ["A", "B", "C", "D"] {
        let mut user = MyUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let filtered = MyUser::find_in_pool(&pool)
        .filter_ne("name", "A")
        .limit(2)
        .offset(1)
        .prepared()
        .all()
        .await
        .expect("all");
    assert_eq!(filtered.len(), 2);

    let unprepared = MyUser::find_in_pool(&pool)
        .filter_gt("id", 0)
        .unprepared()
        .all()
        .await
        .expect("all");
    assert_eq!(unprepared.len(), 4);
}

#[tokio::test]
async fn mysql_raw_filter_requires_allow_unsafe() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <MyUser as Model<sqlx::MySql>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS `{}`", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::MySql, MyUser>(&pool)
        .await
        .expect("sync");

    let mut user = MyUser {
        id: 0,
        name: "Raw".to_string(),
    };
    user.save(&pool).await.expect("save");

    let err = MyUser::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .all()
        .await;
    assert!(err.is_err());

    let ok = MyUser::find_in_pool(&pool)
        .filter_raw("name = 'Raw'")
        .allow_unsafe()
        .all()
        .await;
    assert!(ok.is_ok());
}

#[tokio::test]
async fn mysql_to_sql_uses_placeholders() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let qb = MyUser::find_in_pool(&pool).filter_eq("name", "A");
    let sql = qb.to_sql();
    assert!(sql.contains("?"));
    let update_sql = qb.to_update_sql(&json!({ "name": "B" })).expect("update");
    assert!(update_sql.contains("?"));
}

#[tokio::test]
async fn mysql_stream_api() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <MyUser as Model<sqlx::MySql>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS `{}`", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::MySql, MyUser>(&pool)
        .await
        .expect("sync");

    for name in ["A", "B"] {
        let mut user = MyUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let mut stream = MyUser::find_in_pool(&pool).stream().expect("stream");
    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.expect("row");
        count += 1;
    }
    assert_eq!(count, 2);
}

#[tokio::test]
async fn mysql_filter_like_in() {
    let pool = match get_mysql_pool().await {
        Some(pool) => pool,
        None => return,
    };
    let table = <MyUser as Model<sqlx::MySql>>::table_name();
    let drop_sql = format!("DROP TABLE IF EXISTS `{}`", table);
    sqlx::query(&drop_sql).execute(&pool).await.ok();
    Premix::sync::<sqlx::MySql, MyUser>(&pool)
        .await
        .expect("sync");

    for name in ["Ann", "Bob", "Cara"] {
        let mut user = MyUser {
            id: 0,
            name: name.to_string(),
        };
        user.save(&pool).await.expect("save");
    }

    let like = MyUser::find_in_pool(&pool)
        .filter_like("name", "C%")
        .all()
        .await
        .expect("all");
    assert_eq!(like.len(), 1);

    let ids = vec![like[0].id];
    let in_filter = MyUser::find_in_pool(&pool)
        .filter_in("id", ids)
        .all()
        .await
        .expect("all");
    assert_eq!(in_filter.len(), 1);
}

async fn get_mysql_pool() -> Option<MySqlPool> {
    let db_url = std::env::var("DATABASE_URL").ok()?;
    if !db_url.starts_with("mysql://") {
        return None;
    }
    Some(sqlx::MySqlPool::connect(&db_url).await.ok()?)
}
