use std::sync::atomic::{AtomicI32, Ordering};

use criterion::{Criterion, criterion_group, criterion_main};
use premix_core::{Executor, Model as PremixModel, UpdateResult};
use premix_macros::Model;
use rbatis::RBatis;
use sea_orm::{Database, QuerySelect, Set, TransactionTrait, entity::prelude::*};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use tokio::runtime::Runtime;

// Atomic counter for Unique ID
static INSERT_COUNTER: AtomicI32 = AtomicI32::new(1000);

// --- 1. Premix Model ---
#[derive(Model)]
#[has_many(PostPremix)]
struct UserPremix {
    id: i32,
    name: String,
    #[has_many(PostPremix)]
    #[premix(ignore)]
    posts: Option<Vec<PostPremix>>,
}

#[derive(Model)]
#[belongs_to(UserPremix)]
struct PostPremix {
    id: i32,
    userpremix_id: i32,
    title: String,
}

#[derive(Model)]
struct UserVersioned {
    id: i32,
    name: String,
    version: i32,
}

// --- 2. SeaORM Entity ---
mod user_sea {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "user_seas")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::post_sea::Entity")]
        Post,
    }

    impl Related<super::post_sea::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Post.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod post_sea {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "post_seas")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        pub user_id: i32,
        pub title: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::user_sea::Entity",
            from = "Column::UserId",
            to = "super::user_sea::Column::Id"
        )]
        User,
    }

    impl Related<super::user_sea::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::User.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// --- 3. Rbatis Model ---
#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserRbatis {
    id: i32,
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PostRbatis {
    id: i32,
    user_id: i32,
    title: String,
}

rbatis::crud!(UserRbatis {}, "user_rbatis");
rbatis::crud!(PostRbatis {}, "post_rbatis");

// --- Benchmark Functions ---

async fn setup_db() -> (SqlitePool, sea_orm::DatabaseConnection, RBatis) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create all tables
    sqlx::query("CREATE TABLE user_raws (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    let premix_sql = <UserPremix as PremixModel<sqlx::Sqlite>>::create_table_sql();
    sqlx::query(&premix_sql).execute(&pool).await.unwrap();

    let versioned_sql = <UserVersioned as PremixModel<sqlx::Sqlite>>::create_table_sql();
    sqlx::query(&versioned_sql).execute(&pool).await.unwrap();

    sqlx::query("CREATE TABLE user_seas (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();

    // SeaORM connection
    let sea_db = Database::connect("sqlite::memory:").await.unwrap();
    sea_db
        .execute_unprepared(
            "CREATE TABLE IF NOT EXISTS user_seas (id INTEGER PRIMARY KEY, name TEXT)",
        )
        .await
        .unwrap();
    sea_db
        .execute_unprepared(
            "CREATE TABLE IF NOT EXISTS post_seas (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT)",
        )
        .await
        .unwrap();

    // Rbatis connection
    let rb = RBatis::new();
    rb.init(rbdc_sqlite::driver::SqliteDriver {}, "sqlite::memory:")
        .unwrap();
    rb.exec(
        "CREATE TABLE IF NOT EXISTS user_rbatis (id INTEGER PRIMARY KEY, name TEXT)",
        vec![],
    )
    .await
    .unwrap();
    rb.exec(
        "CREATE TABLE IF NOT EXISTS post_rbatis (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT)",
        vec![],
    )
    .await
    .unwrap();

    (pool, sea_db, rb)
}

fn benchmark_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    let mut group = c.benchmark_group("Insert (4-Way)");

    // 1. Raw SQLx
    group.bench_function("sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind("Test")
                .execute(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Premix ORM
    group.bench_function("premix_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut user = UserPremix {
                id,
                name: "Test".to_string(),
                posts: None,
            };
            <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut user, &pool)
                .await
                .unwrap();
        })
    });

    // 3. SeaORM
    group.bench_function("sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let user = user_sea::ActiveModel {
                id: Set(id),
                name: Set("Test".to_string()),
            };
            user_sea::Entity::insert(user).exec(&sea_db).await.unwrap();
        })
    });

    // 4. Rbatis
    group.bench_function("rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let user = UserRbatis {
                id,
                name: "Test".to_string(),
            };
            UserRbatis::insert(&rb, &user).await.unwrap();
        })
    });

    group.finish();
}

fn benchmark_select(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    // Seed Data
    rt.block_on(async {
        sqlx::query("INSERT INTO user_raws (id, name) VALUES (1, 'Data')")
            .execute(&pool)
            .await
            .unwrap();
        let _ = UserPremix {
            id: 1,
            name: "Data".to_string(),
            posts: None,
        }
        .save(&pool)
        .await;
        let user = user_sea::ActiveModel {
            id: Set(1),
            name: Set("Data".to_string()),
        };
        let _ = user_sea::Entity::insert(user).exec(&sea_db).await;
        let rb_user = UserRbatis {
            id: 1,
            name: "Data".to_string(),
        };
        let _ = UserRbatis::insert(&rb, &rb_user).await;
    });

    let mut group = c.benchmark_group("Select (4-Way)");

    // 1. Raw SQLx
    group.bench_function("sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let _row = sqlx::query("SELECT * FROM user_raws WHERE id = $1")
                .bind(1)
                .fetch_one(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Premix ORM
    group.bench_function("premix_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let _user = <UserPremix as PremixModel<sqlx::Sqlite>>::find_by_id(&pool, 1)
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
            use rbs::value;
            let _users: Vec<UserRbatis> = UserRbatis::select_by_map(&rb, value! {"id": 1})
                .await
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_bulk_select(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    rt.block_on(async {
        for i in 1..=100 {
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(i)
                .bind(format!("User {}", i))
                .execute(&pool)
                .await
                .unwrap();

            let mut u = UserPremix {
                id: i,
                name: format!("User {}", i),
                posts: None,
            };
            let _ = <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut u, &pool).await;

            let user = user_sea::ActiveModel {
                id: Set(i),
                name: Set(format!("User {}", i)),
            };
            let _ = user_sea::Entity::insert(user).exec(&sea_db).await;

            let rb_user = UserRbatis {
                id: i,
                name: format!("User {}", i),
            };
            let _ = UserRbatis::insert(&rb, &rb_user).await;
        }
    });

    let mut group = c.benchmark_group("Bulk Select 100 Rows");

    // 1. Raw SQLx
    group.bench_function("sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let _rows = sqlx::query("SELECT * FROM user_raws LIMIT 100")
                .fetch_all(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Premix ORM
    group.bench_function("premix_orm_manual_map", |b| {
        b.to_async(&rt).iter(|| async {
            let rows = sqlx::query_as::<sqlx::sqlite::Sqlite, UserPremix>(
                "SELECT * FROM userpremixs LIMIT 100",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
            assert_eq!(rows.len(), 100);
        })
    });

    // 3. SeaORM
    group.bench_function("sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let _users = user_sea::Entity::find()
                .limit(100)
                .all(&sea_db)
                .await
                .unwrap();
        })
    });

    // 4. Rbatis
    group.bench_function("rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let _users: Vec<UserRbatis> = rb
                .query_decode("SELECT * FROM user_rbatis LIMIT 100", vec![])
                .await
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_relation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    rt.block_on(async {
        let _sql = <UserPremix as PremixModel<sqlx::Sqlite>>::create_table_sql();
        let _sql2 = <PostPremix as PremixModel<sqlx::Sqlite>>::create_table_sql();
        sqlx::query(&_sql).execute(&pool).await.unwrap();
        sqlx::query(&_sql2).execute(&pool).await.unwrap();

        sqlx::query("CREATE TABLE post_raws (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let mut user = UserPremix {
            id: 1,
            name: "Boss".to_string(),
            posts: None,
        };
        <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut user, &pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO user_raws (id, name) VALUES (1, 'Boss')")
            .execute(&pool)
            .await
            .unwrap();

        for i in 1..=10 {
            let mut post = PostPremix {
                id: i,
                userpremix_id: 1,
                title: format!("Post {}", i),
            };
            <PostPremix as PremixModel<sqlx::Sqlite>>::save(&mut post, &pool)
                .await
                .unwrap();

            sqlx::query("INSERT INTO post_raws (id, user_id, title) VALUES ($1, $2, $3)")
                .bind(i)
                .bind(1)
                .bind(format!("Post {}", i))
                .execute(&pool)
                .await
                .unwrap();
        }

        // Seed SeaORM
        let user = user_sea::ActiveModel {
            id: Set(1),
            name: Set("Boss".to_string()),
        };
        user_sea::Entity::insert(user).exec(&sea_db).await.unwrap();
        for i in 1..=10 {
            let post = post_sea::ActiveModel {
                id: Set(i),
                user_id: Set(1),
                title: Set(format!("Post {}", i)),
            };
            post_sea::Entity::insert(post).exec(&sea_db).await.unwrap();
        }

        // Seed Rbatis
        let u = UserRbatis {
            id: 1,
            name: "Boss".to_string(),
        };
        UserRbatis::insert(&rb, &u).await.unwrap();
        for i in 1..=10 {
            let p = PostRbatis {
                id: i,
                user_id: 1,
                title: format!("Post {}", i),
            };
            PostRbatis::insert(&rb, &p).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("Relation (1 User -> 10 Posts)");

    group.bench_function("sqlx_raw_join", |b| {
        b.to_async(&rt).iter(|| async {
            let _rows = sqlx::query("SELECT u.*, p.* FROM user_raws u JOIN post_raws p ON u.id = p.user_id WHERE u.id = 1")
                .fetch_all(&pool)
                .await
                .unwrap();
        })
    });

    group.bench_function("premix_relation", |b| {
        b.to_async(&rt).iter(|| async {
            let user = <UserPremix as PremixModel<sqlx::Sqlite>>::find_by_id(&pool, 1)
                .await
                .unwrap()
                .unwrap();
            let _posts = user
                .postpremixs_lazy::<_, sqlx::Sqlite>(&pool)
                .await
                .unwrap();
        })
    });

    group.bench_function("premix_eager", |b| {
        b.to_async(&rt).iter(|| async {
            let _users = <UserPremix as PremixModel<sqlx::Sqlite>>::find_in_pool(&pool)
                .include("posts")
                .filter("id = 1")
                .limit(1)
                .all()
                .await
                .unwrap();
        })
    });

    // 4. SeaORM (Relation)
    group.bench_function("sea_orm_relation", |b| {
        b.to_async(&rt).iter(|| async {
            let _users: Vec<(user_sea::Model, Vec<post_sea::Model>)> =
                user_sea::Entity::find_by_id(1)
                    .find_with_related(post_sea::Entity)
                    .all(&sea_db)
                    .await
                    .unwrap();
        })
    });

    // 5. Rbatis (Manual Relation)
    group.bench_function("rbatis_manual", |b| {
        b.to_async(&rt).iter(|| async {
            use rbs::value;
            let user: Option<UserRbatis> = UserRbatis::select_by_map(&rb, value! {"id": 1})
                .await
                .unwrap()
                .into_iter()
                .next();
            if let Some(u) = user {
                let _posts: Vec<PostRbatis> =
                    PostRbatis::select_by_map(&rb, value! {"user_id": u.id})
                        .await
                        .unwrap();
            }
        })
    });

    group.finish();
}

fn benchmark_bulk_relation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    rt.block_on(async {
        let _ = sqlx::query(&<PostPremix as PremixModel<sqlx::Sqlite>>::create_table_sql()).execute(&pool).await;
        let _ = sqlx::query("CREATE TABLE IF NOT EXISTS post_raws (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT)").execute(&pool).await;

        for i in 1..=50 {
            let mut u = UserPremix { id: i, name: format!("User {}", i), posts: None };
            <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut u, &pool).await.unwrap();
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)").bind(i).bind(format!("User {}", i)).execute(&pool).await.unwrap();

            for j in 1..=10 {
                let pid = (i * 1000) + j;
                let mut p = PostPremix { id: pid, userpremix_id: i, title: format!("Post {}", pid) };
                <PostPremix as PremixModel<sqlx::Sqlite>>::save(&mut p, &pool).await.unwrap();
                sqlx::query("INSERT INTO post_raws (id, user_id, title) VALUES ($1, $2, $3)").bind(pid).bind(i).bind(format!("Post {}", pid)).execute(&pool).await.unwrap();
            }

            // Seed SeaORM
            let user_s = user_sea::ActiveModel { id: Set(i), name: Set(format!("User {}", i)) };
            user_sea::Entity::insert(user_s).exec(&sea_db).await.unwrap();
            for j in 1..=10 {
                let pid = (i * 1000) + j;
                let post_s = post_sea::ActiveModel { id: Set(pid), user_id: Set(i), title: Set(format!("Post {}", pid)) };
                post_sea::Entity::insert(post_s).exec(&sea_db).await.unwrap();
            }

            // Seed Rbatis
            let user_r = UserRbatis { id: i, name: format!("User {}", i) };
            UserRbatis::insert(&rb, &user_r).await.unwrap();
            for j in 1..=10 {
                let pid = (i * 1000) + j;
                let post_r = PostRbatis { id: pid, user_id: i, title: format!("Post {}", pid) };
                PostRbatis::insert(&rb, &post_r).await.unwrap();
            }
        }
    });

    let mut group = c.benchmark_group("Bulk Relation (50 Users -> 500 Posts)");

    group.bench_function("sqlx_raw_join", |b| {
        b.to_async(&rt).iter(|| async {
            let _rows = sqlx::query(
                "SELECT u.*, p.* FROM user_raws u JOIN post_raws p ON u.id = p.user_id",
            )
            .fetch_all(&pool)
            .await
            .unwrap();
        })
    });

    group.bench_function("lazy_loading_100_users", |b| {
        b.to_async(&rt).iter(|| async {
            let users = sqlx::query_as::<_, UserPremix>("SELECT * FROM userpremixs")
                .fetch_all(&pool)
                .await
                .unwrap();
            for user in users {
                let _posts = user
                    .postpremixs_lazy::<_, sqlx::Sqlite>(&pool)
                    .await
                    .unwrap();
            }
        })
    });

    group.bench_function("premix_eager_batch", |b| {
        b.to_async(&rt).iter(|| async {
            let _users = <UserPremix as PremixModel<sqlx::Sqlite>>::find_in_pool(&pool)
                .include("posts")
                .all()
                .await
                .unwrap();
        })
    });

    // 4. SeaORM (Relation Bulk)
    group.bench_function("sea_orm_relation_bulk", |b| {
        b.to_async(&rt).iter(|| async {
            let _users: Vec<(user_sea::Model, Vec<post_sea::Model>)> = user_sea::Entity::find()
                .find_with_related(post_sea::Entity)
                .all(&sea_db)
                .await
                .unwrap();
        })
    });

    // 5. Rbatis (Bulk Manual)
    group.bench_function("rbatis_bulk_manual", |b| {
        b.to_async(&rt).iter(|| async {
            // 1. Fetch Users
            let _users: Vec<UserRbatis> = rb
                .query_decode("SELECT * FROM user_rbatis", vec![])
                .await
                .unwrap();

            // 2. Fetch Posts (In-App Join)
            let _posts: Vec<PostRbatis> = rb
                .query_decode("SELECT * FROM post_rbatis", vec![])
                .await
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_update_delete(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    rt.block_on(async {
        // Seed 1 record for update/delete
        sqlx::query("CREATE TABLE IF NOT EXISTS user_raws (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();
    });

    let mut group = c.benchmark_group("Modify Operations");

    // 1. Update Benchmark (Baseline)
    group.bench_function("update_sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind("Original")
                .execute(&pool)
                .await
                .unwrap();

            sqlx::query("UPDATE user_raws SET name = $1 WHERE id = $2")
                .bind("Updated")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();
        })
    });

    // 2. Update Benchmark (Premix)
    group.bench_function("update_premix", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut u = UserPremix {
                id,
                name: "Original".to_string(),
                posts: None,
            };
            <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut u, &pool)
                .await
                .unwrap();

            u.name = "Updated".to_string();
            let result =
                <UserPremix as PremixModel<sqlx::Sqlite>>::update(&mut u, Executor::Pool(&pool))
                    .await
                    .unwrap();
            assert_eq!(result, UpdateResult::Success);
        })
    });

    // 3. Delete Benchmark (Baseline)
    group.bench_function("delete_sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind("To Delete")
                .execute(&pool)
                .await
                .unwrap();

            sqlx::query("DELETE FROM user_raws WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();
        })
    });

    // 4. Delete Benchmark (Premix)
    group.bench_function("delete_premix", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut u = UserPremix {
                id,
                name: "To Delete".to_string(),
                posts: None,
            };
            <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut u, &pool)
                .await
                .unwrap();

            <UserPremix as PremixModel<sqlx::Sqlite>>::delete(&mut u, Executor::Pool(&pool))
                .await
                .unwrap();
        })
    });

    // 5. Update Benchmark (SeaORM)
    group.bench_function("update_sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let user = user_sea::ActiveModel {
                id: Set(id),
                name: Set("Original".to_string()),
            };
            user_sea::Entity::insert(user).exec(&sea_db).await.unwrap();

            let user_model = user_sea::Entity::find_by_id(id)
                .one(&sea_db)
                .await
                .unwrap()
                .unwrap();
            let mut active: user_sea::ActiveModel = user_model.into();
            active.name = Set("Updated".to_string());
            active.update(&sea_db).await.unwrap();
        })
    });

    // 6. Update Benchmark (Rbatis)
    group.bench_function("update_rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let u = UserRbatis {
                id,
                name: "Original".to_string(),
            };
            UserRbatis::insert(&rb, &u).await.unwrap();

            rb.exec(
                "UPDATE user_rbatis SET name = ? WHERE id = ?",
                vec![rbs::value!("Updated"), rbs::value!(id)],
            )
            .await
            .unwrap();
        })
    });

    // 7. Delete Benchmark (SeaORM)
    group.bench_function("delete_sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let user = user_sea::ActiveModel {
                id: Set(id),
                name: Set("To Delete".to_string()),
            };
            user_sea::Entity::insert(user).exec(&sea_db).await.unwrap();

            user_sea::Entity::delete_by_id(id)
                .exec(&sea_db)
                .await
                .unwrap();
        })
    });

    // 8. Delete Benchmark (Rbatis)
    group.bench_function("delete_rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let u = UserRbatis {
                id,
                name: "To Delete".to_string(),
            };
            UserRbatis::insert(&rb, &u).await.unwrap();

            rb.exec(
                "DELETE FROM user_rbatis WHERE id = ?",
                vec![rbs::value!(id)],
            )
            .await
            .unwrap();
        })
    });

    group.finish();
}

fn benchmark_transactions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, sea_db, rb) = rt.block_on(setup_db());

    let mut group = c.benchmark_group("Transactions");

    // 1. Raw SQL Transaction
    group.bench_function("tx_sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tx = pool.begin().await.unwrap();
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);

            sqlx::query("INSERT INTO user_raws (id, name) VALUES ($1, $2)")
                .bind(id)
                .bind("Tx Raw")
                .execute(&mut *tx)
                .await
                .unwrap();

            tx.commit().await.unwrap();
        })
    });

    // 2. Premix Transaction
    group.bench_function("tx_premix", |b| {
        b.to_async(&rt).iter(|| async {
            let mut tx = pool.begin().await.unwrap();
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut u = UserPremix {
                id,
                name: "Tx Premix".to_string(),
                posts: None,
            };

            <UserPremix as PremixModel<sqlx::Sqlite>>::save(&mut u, &mut *tx)
                .await
                .unwrap();

            tx.commit().await.unwrap();
        })
    });

    // 3. SeaORM Transaction
    group.bench_function("tx_sea_orm", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            sea_db
                .transaction::<_, (), sea_orm::DbErr>(|tx| {
                    Box::pin(async move {
                        let user = user_sea::ActiveModel {
                            id: Set(id),
                            name: Set("Tx Sea".to_string()),
                        };
                        user_sea::Entity::insert(user).exec(tx).await?;
                        Ok(())
                    })
                })
                .await
                .unwrap();
        })
    });

    // 4. Rbatis Transaction
    group.bench_function("tx_rbatis", |b| {
        b.to_async(&rt).iter(|| async {
            let tx = rb.acquire_begin().await.unwrap();
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let u = UserRbatis {
                id,
                name: "Tx Rbatis".to_string(),
            };
            UserRbatis::insert(&tx, &u).await.unwrap();
            tx.commit().await.unwrap();
        })
    });

    group.finish();
}

fn benchmark_bulk_ops(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, _, _) = rt.block_on(setup_db());

    // Setup 1000 users
    rt.block_on(async {
        let mut tx = pool.begin().await.unwrap();
        for i in 0..1000 {
            sqlx::query("INSERT INTO userpremixs (name) VALUES ($1)")
                .bind(format!("User {}", i))
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    });

    let mut group = c.benchmark_group("Bulk Operations (1000 Rows)");
    group.sample_size(10);

    // 1. Loop Update
    group.bench_function("loop_update_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let users = <UserPremix as PremixModel<sqlx::Sqlite>>::find_in_pool(&pool)
                .all()
                .await
                .unwrap();
            for mut user in users {
                user.name = "Updated Loop".to_string();
                <UserPremix as PremixModel<sqlx::Sqlite>>::update(&mut user, Executor::Pool(&pool))
                    .await
                    .unwrap();
            }
        })
    });

    // 2. Bulk Update
    group.bench_function("bulk_update_1000", |b| {
        b.to_async(&rt).iter(|| async {
            <UserPremix as PremixModel<sqlx::Sqlite>>::find_in_pool(&pool)
                .update(serde_json::json!({ "name": "Updated Bulk" }))
                .await
                .unwrap();
        })
    });

    group.finish();
}

fn benchmark_optimistic_locking(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (pool, _sea_db, _rb) = rt.block_on(setup_db());

    let mut group = c.benchmark_group("Optimistic Locking");

    // 1. Raw SQL Optimistic Lock
    group.bench_function("opt_lock_sqlx_raw", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            // Insert
            sqlx::query("INSERT INTO userversioneds (id, name, version) VALUES ($1, $2, $3)")
                .bind(id).bind("Original").bind(1)
                .execute(&pool).await.unwrap();

            // Update (Check Version)
            let result = sqlx::query("UPDATE userversioneds SET name = $1, version = version + 1 WHERE id = $2 AND version = $3")
                .bind("Updated")
                .bind(id)
                .bind(1)
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(result.rows_affected(), 1);
        })
    });

    // 2. Premix Optimistic Lock
    group.bench_function("opt_lock_premix", |b| {
        b.to_async(&rt).iter(|| async {
            let id = INSERT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let mut u = UserVersioned {
                id,
                name: "Original".to_string(),
                version: 1,
            };
            <UserVersioned as PremixModel<sqlx::Sqlite>>::save(&mut u, &pool)
                .await
                .unwrap();

            u.name = "Updated".to_string();
            let result =
                <UserVersioned as PremixModel<sqlx::Sqlite>>::update(&mut u, Executor::Pool(&pool))
                    .await
                    .unwrap();
            assert_eq!(result, UpdateResult::Success);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_insert,
    benchmark_select,
    benchmark_bulk_select,
    benchmark_relation,
    benchmark_bulk_relation,
    benchmark_update_delete,
    benchmark_transactions,
    benchmark_optimistic_locking,
    benchmark_bulk_ops
);
criterion_main!(benches);
