// I/O Benchmark - Real Postgres Network Latency Comparison
// Compare: Premix ORM vs SeaORM vs Rbatis vs Raw SQL

#[cfg(feature = "postgres")]
use std::env;
#[cfg(feature = "postgres")]
use std::sync::atomic::{AtomicI32, Ordering};
#[cfg(feature = "postgres")]
use std::time::Duration;

#[cfg(feature = "postgres")]
use criterion::{Criterion, criterion_group, criterion_main};
#[cfg(feature = "postgres")]
use premix_core::Model as PremixModel;
#[cfg(feature = "postgres")]
use premix_macros::Model;
#[cfg(feature = "postgres")]
use sea_orm::{Database, Set, entity::prelude::*};
#[cfg(feature = "postgres")]
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "postgres")]
use tokio::runtime::Runtime;

#[cfg(feature = "postgres")]
static INSERT_COUNTER: AtomicI32 = AtomicI32::new(1);

// ============ Premix Model ============
#[cfg(feature = "postgres")]
#[derive(Model, Clone, Debug)]
struct UserIO {
    id: i32,
    name: String,
}

// ============ SeaORM Entity ============
#[cfg(feature = "postgres")]
mod user_sea {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user_seas")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// ============ Rbatis Model ============
#[cfg(feature = "postgres")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct UserRbatis {
    id: Option<i32>,
    name: Option<String>,
}
#[cfg(feature = "postgres")]
rbatis::crud!(UserRbatis {}, "user_rbatises");

// ============ Setup ============
#[cfg(feature = "postgres")]
async fn setup_postgres() -> (
    sqlx::Pool<sqlx::Postgres>,
    sea_orm::DatabaseConnection,
    rbatis::RBatis,
) {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:admin123@localhost:5432/premix_bench".to_string());

    // SQLx Pool (for Premix + Raw SQL)
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // SeaORM Connection
    let sea_db = Database::connect(&database_url)
        .await
        .expect("SeaORM connection failed");

    // Rbatis Connection
    let rb = rbatis::RBatis::new();
    rb.init(rbdc_pg::driver::PgDriver {}, &database_url)
        .expect("Rbatis connection failed");

    // Create tables (separate queries for Postgres)
    sqlx::query("CREATE TABLE IF NOT EXISTS userios (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS user_seas (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS user_rbatises (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS user_raws (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    // Truncate for clean state
    sqlx::query("TRUNCATE TABLE userios RESTART IDENTITY")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("TRUNCATE TABLE user_seas RESTART IDENTITY")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("TRUNCATE TABLE user_rbatises RESTART IDENTITY")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("TRUNCATE TABLE user_raws RESTART IDENTITY")
        .execute(&pool)
        .await
        .ok();

    (pool, sea_db, rb)
}

// ============ INSERT Benchmark ============
#[cfg(feature = "postgres")]
fn benchmark_io_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_postgres());

    let mut group = c.benchmark_group("IO Insert (Postgres)");
    group.sample_size(50);

    // 1. Raw SQL
    group.bench_function("raw_sql", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::Relaxed);
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind(format!("Raw User {}", id))
                .execute(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Premix ORM
    group.bench_function("premix", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::Relaxed);
            let mut u = UserIO {
                id,
                name: format!("Premix User {}", id),
            };
            <UserIO as PremixModel<sqlx::Postgres>>::save_ultra(&mut u, &pool)
                .await
                .unwrap();
        })
    });

    // 3. SeaORM
    group.bench_function("sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::Relaxed);
            let user = user_sea::ActiveModel {
                id: Set(id),
                name: Set(format!("SeaORM User {}", id)),
            };
            user_sea::Entity::insert(user).exec(&sea_db).await.unwrap();
        })
    });

    // 4. Rbatis
    group.bench_function("rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::Relaxed);
            let user = UserRbatis {
                id: Some(id),
                name: Some(format!("Rbatis User {}", id)),
            };
            UserRbatis::insert(&rb, &user).await.unwrap();
        })
    });

    group.finish();
}

// ============ SELECT Benchmark ============
#[cfg(feature = "postgres")]
fn benchmark_io_select(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_postgres());

    // Seed data
    rt.block_on(async {
        sqlx::query("INSERT INTO userios (id, name) VALUES (1, 'Select Me')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO user_seas (id, name) VALUES (1, 'Select Me')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO user_rbatises (id, name) VALUES (1, 'Select Me')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO user_raws (id, name) VALUES (1, 'Select Me')")
            .execute(&pool)
            .await
            .unwrap();
    });

    let mut group = c.benchmark_group("IO Select (Postgres)");
    group.sample_size(100);

    // 1. Raw SQL
    group.bench_function("raw_sql", |b| {
        b.to_async(&rt).iter(|| async {
            let _: (i32, String) = sqlx::query_as("SELECT id, name FROM user_raws WHERE id = $1")
                .bind(1)
                .fetch_one(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Premix ORM
    group.bench_function("premix", |b| {
        b.to_async(&rt).iter(|| async {
            let _user = <UserIO as PremixModel<sqlx::Postgres>>::find_by_id(&pool, 1)
                .await
                .unwrap();
        })
    });

    // 3. SeaORM
    group.bench_function("sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let _user = user_sea::Entity::find_by_id(1).one(&sea_db).await.unwrap();
        })
    });

    // 4. Rbatis
    group.bench_function("rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let _user: Vec<UserRbatis> = rb
                .query_decode(
                    "SELECT * FROM user_rbatises WHERE id = $1",
                    vec![rbs::value!(1)],
                )
                .await
                .unwrap();
        })
    });

    group.finish();
}

// ============ Concurrency Benchmark ============
#[cfg(feature = "postgres")]
fn benchmark_io_concurrency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, _sea_db, _rb) = rt.block_on(setup_postgres());

    // Seed
    rt.block_on(async {
        sqlx::query(
            "INSERT INTO userios (id, name) VALUES (1, 'Concurrent') ON CONFLICT DO NOTHING",
        )
        .execute(&pool)
        .await
        .ok();
    });

    let mut group = c.benchmark_group("IO Concurrency (Postgres)");
    group.sample_size(20);

    group.bench_function("premix_10_concurrent_selects", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = vec![];
            for _ in 0..10 {
                let p = pool.clone();
                tasks.push(tokio::spawn(async move {
                    <UserIO as PremixModel<sqlx::Postgres>>::find_by_id(&p, 1)
                        .await
                        .unwrap();
                }));
            }
            for t in tasks {
                t.await.unwrap();
            }
        })
    });

    group.bench_function("raw_sql_10_concurrent_selects", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tasks = vec![];
            for _ in 0..10 {
                let p = pool.clone();
                tasks.push(tokio::spawn(async move {
                    let _: (i32, String) =
                        sqlx::query_as("SELECT id, name FROM userios WHERE id = $1")
                            .bind(1)
                            .fetch_one(&p)
                            .await
                            .unwrap();
                }));
            }
            for t in tasks {
                t.await.unwrap();
            }
        })
    });

    group.finish();
}

#[cfg(feature = "postgres")]
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10));
    targets = benchmark_io_insert, benchmark_io_select, benchmark_io_concurrency
);

#[cfg(feature = "postgres")]
criterion_main!(benches);

#[cfg(not(feature = "postgres"))]
fn main() {
    println!("Please enable 'postgres' feature to run this benchmark:");
    println!("  cargo bench --bench io_benchmark --features postgres");
}
