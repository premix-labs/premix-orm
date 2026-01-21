use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use premix_orm::prelude::*;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Model, Debug, Clone)]
struct User {
    id: i32,
    name: String,
    age: i32,
}

async fn init_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:?cache=shared")
        .await
        .expect("connect sqlite memory");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("create table");

    sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await
        .expect("clear table");

    sqlx::query("INSERT INTO users (name, age) VALUES (?, ?)")
        .bind("Alice")
        .bind(42)
        .execute(&pool)
        .await
        .expect("seed row");

    Premix::sync::<sqlx::Sqlite, User>(&pool)
        .await
        .expect("sync schema");

    pool
}

fn bench_premix_vs_sqlx(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let pool = rt.block_on(init_pool());
    let pool = Arc::new(pool);

    let raw_pool = pool.clone();
    c.bench_function("raw_sqlx_fetch_one", |b| {
        b.to_async(&rt).iter(|| {
            let pool = raw_pool.clone();
            async move {
                let user: User = sqlx::query_as::<_, User>("SELECT id, name, age FROM users WHERE id = ?")
                    .bind(1)
                    .fetch_one(&*pool)
                    .await
                    .expect("raw fetch");
                std::hint::black_box(user);
            }
        })
    });

    let premix_pool = pool.clone();
    c.bench_function("premix_find_fetch_one", |b| {
        b.to_async(&rt).iter(|| {
            let pool = premix_pool.clone();
            async move {
                let mut users = User::find_in_pool(&*pool)
                    .filter_eq("id", 1)
                    .limit(1)
                    .all()
                    .await
                    .expect("premix fetch");
                let user = users.pop().expect("one row");
                std::hint::black_box(user);
            }
        })
    });
}

criterion_group!(benches, bench_premix_vs_sqlx);
criterion_main!(benches);
