use std::error::Error;

use sqlx::{Database, Executor, Pool};

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct AppliedMigration {
    version: String,
}

pub struct Migrator<DB: Database> {
    pool: Pool<DB>,
}

impl<DB: Database> Migrator<DB> {
    pub fn new(pool: Pool<DB>) -> Self {
        Self { pool }
    }
}

// Specialized implementations for SQLite (Feature-gated or trait-based later)
// For Version 1, we'll try to use generic Executor where possible,
// but creating the migrations table often requires dialect specific SQL.

#[cfg(feature = "sqlite")]
impl Migrator<sqlx::Sqlite> {
    pub async fn run(&self, migrations: Vec<Migration>) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        // 1. Ensure Table Exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _premix_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&mut *tx)
        .await?;

        // 2. Get Applied Versions
        let applied_versions: Vec<String> = sqlx::query_as::<_, AppliedMigration>(
            "SELECT version FROM _premix_migrations ORDER BY version ASC",
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|m| m.version)
        .collect();

        // 3. Filter Pending
        for migration in migrations {
            if !applied_versions.contains(&migration.version) {
                tracing::info!(
                    operation = "migration_apply",
                    version = %migration.version,
                    name = %migration.name,
                    "premix migration"
                );
                println!(
                    "🚚 Applying migration: {} - {}",
                    migration.version, migration.name
                );

                // Execute UP SQL
                tx.execute(migration.up_sql.as_str()).await?;

                // Record Version
                sqlx::query("INSERT INTO _premix_migrations (version, name) VALUES (?, ?)")
                    .bind(&migration.version)
                    .bind(&migration.name)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn rollback_last(&self, migrations: Vec<Migration>) -> Result<bool, Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _premix_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&mut *tx)
        .await?;

        let versions: Vec<String> = migrations.iter().map(|m| m.version.clone()).collect();
        if versions.is_empty() {
            tx.commit().await?;
            return Ok(false);
        }

        let placeholders = vec!["?"; versions.len()].join(", ");
        let sql = format!(
            "SELECT version FROM _premix_migrations WHERE version IN ({}) ORDER BY version DESC LIMIT 1",
            placeholders
        );
        let mut query = sqlx::query_scalar::<_, String>(&sql);
        for version in &versions {
            query = query.bind(version);
        }
        let last = query.fetch_optional(&mut *tx).await?;

        let Some(last) = last else {
            tx.commit().await?;
            return Ok(false);
        };

        let migration = migrations
            .into_iter()
            .find(|m| m.version == last)
            .ok_or_else(|| format!("Migration {} not found.", last))?;

        if migration.down_sql.trim().is_empty() {
            return Err("Down migration is empty.".into());
        }

        tracing::info!(
            operation = "migration_rollback",
            version = %migration.version,
            name = %migration.name,
            "premix migration"
        );
        tx.execute(migration.down_sql.as_str()).await?;
        sqlx::query("DELETE FROM _premix_migrations WHERE version = ?")
            .bind(&migration.version)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(true)
    }
}

#[cfg(feature = "postgres")]
impl Migrator<sqlx::Postgres> {
    pub async fn run(&self, migrations: Vec<Migration>) -> Result<(), Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        // 1. Ensure Table Exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _premix_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&mut *tx)
        .await?;

        // 2. Get Applied Versions
        let applied_versions: Vec<String> = sqlx::query_as::<_, AppliedMigration>(
            "SELECT version FROM _premix_migrations ORDER BY version ASC",
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|m| m.version)
        .collect();

        // 3. Filter Pending
        for migration in migrations {
            if !applied_versions.contains(&migration.version) {
                tracing::info!(
                    operation = "migration_apply",
                    version = %migration.version,
                    name = %migration.name,
                    "premix migration"
                );
                println!(
                    "🚚 Applying migration: {} - {}",
                    migration.version, migration.name
                );

                // Execute UP SQL
                // Note: splitting by ; might be needed for multiple statements in one file
                // But for MVP we assume sqlx can handle the string block or user separates properly.
                // sqlx::execute only runs the first statement for some drivers,
                // but Executor::execute roughly maps to running the query.
                // For safety in Postgres with multiple statements, simple Executor::execute might fail if not wrapped or specific support.
                // We'll trust user provides valid script block for now.
                tx.execute(migration.up_sql.as_str()).await?;

                // Record Version
                sqlx::query("INSERT INTO _premix_migrations (version, name) VALUES ($1, $2)")
                    .bind(&migration.version)
                    .bind(&migration.name)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn rollback_last(&self, migrations: Vec<Migration>) -> Result<bool, Box<dyn Error>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _premix_migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&mut *tx)
        .await?;

        let versions: Vec<String> = migrations.iter().map(|m| m.version.clone()).collect();
        if versions.is_empty() {
            tx.commit().await?;
            return Ok(false);
        }

        let last = sqlx::query_scalar::<_, String>(
            "SELECT version FROM _premix_migrations WHERE version = ANY($1) ORDER BY version DESC LIMIT 1",
        )
        .bind(&versions)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(last) = last else {
            tx.commit().await?;
            return Ok(false);
        };

        let migration = migrations
            .into_iter()
            .find(|m| m.version == last)
            .ok_or_else(|| format!("Migration {} not found.", last))?;

        if migration.down_sql.trim().is_empty() {
            return Err("Down migration is empty.".into());
        }

        tracing::info!(
            operation = "migration_rollback",
            version = %migration.version,
            name = %migration.name,
            "premix migration"
        );
        tx.execute(migration.down_sql.as_str()).await?;
        sqlx::query("DELETE FROM _premix_migrations WHERE version = $1")
            .bind(&migration.version)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "postgres")]
    use std::time::{SystemTime, UNIX_EPOCH};

    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

    #[cfg(feature = "postgres")]
    async fn pg_pool_or_skip() -> Option<sqlx::Pool<sqlx::Postgres>> {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:admin123@localhost:5432/premix_bench".to_string()
        });
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&db_url)
            .await
        {
            Ok(pool) => Some(pool),
            Err(_) => None,
        }
    }

    #[tokio::test]
    async fn sqlite_migrator_applies_pending_once() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let migrator = Migrator::new(pool.clone());

        let migrations = vec![Migration {
            version: "20260101000000".to_string(),
            name: "create_users".to_string(),
            up_sql: "CREATE TABLE users (id INTEGER PRIMARY KEY);".to_string(),
            down_sql: "DROP TABLE users;".to_string(),
        }];

        migrator.run(migrations.clone()).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        migrator.run(migrations).await.unwrap();
        let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_after, 1);
    }

    #[tokio::test]
    async fn sqlite_migrator_applies_multiple() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let migrator = Migrator::new(pool.clone());

        let migrations = vec![
            Migration {
                version: "20260103000000".to_string(),
                name: "create_a".to_string(),
                up_sql: "CREATE TABLE a (id INTEGER PRIMARY KEY);".to_string(),
                down_sql: "DROP TABLE a;".to_string(),
            },
            Migration {
                version: "20260104000000".to_string(),
                name: "create_b".to_string(),
                up_sql: "CREATE TABLE b (id INTEGER PRIMARY KEY);".to_string(),
                down_sql: "DROP TABLE b;".to_string(),
            },
        ];

        migrator.run(migrations).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn sqlite_migrator_rolls_back_last() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let migrator = Migrator::new(pool.clone());

        let migrations = vec![
            Migration {
                version: "20260103000000".to_string(),
                name: "create_a".to_string(),
                up_sql: "CREATE TABLE a (id INTEGER PRIMARY KEY);".to_string(),
                down_sql: "DROP TABLE a;".to_string(),
            },
            Migration {
                version: "20260104000000".to_string(),
                name: "create_b".to_string(),
                up_sql: "CREATE TABLE b (id INTEGER PRIMARY KEY);".to_string(),
                down_sql: "DROP TABLE b;".to_string(),
            },
        ];

        migrator.run(migrations.clone()).await.unwrap();
        let rolled_back = migrator.rollback_last(migrations).await.unwrap();
        assert!(rolled_back);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn sqlite_migrator_rolls_back_on_error() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let migrator = Migrator::new(pool.clone());

        let migrations = vec![Migration {
            version: "20260102000000".to_string(),
            name: "bad_sql".to_string(),
            up_sql: "CREATE TABLE broken (id INTEGER PRIMARY KEY); INVALID SQL".to_string(),
            down_sql: "DROP TABLE broken;".to_string(),
        }];

        let err = migrator.run(migrations).await.unwrap_err();
        assert!(err.to_string().contains("syntax"));

        let table: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='_premix_migrations'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(table.is_none());
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn postgres_migrator_applies_pending_once() {
        let Some(pool) = pg_pool_or_skip().await else {
            return;
        };
        let migrator = Migrator::new(pool.clone());

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let version = format!("20260101{:020}", suffix);
        let table_name = format!("premix_mig_test_{}", suffix);
        let migrations = vec![Migration {
            version: version.clone(),
            name: "create_test_table".to_string(),
            up_sql: format!("CREATE TABLE {} (id SERIAL PRIMARY KEY);", table_name),
            down_sql: format!("DROP TABLE {};", table_name),
        }];

        migrator.run(migrations.clone()).await.unwrap();

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations WHERE version = $1")
                .bind(&version)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);

        migrator.run(migrations).await.unwrap();
        let count_after: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations WHERE version = $1")
                .bind(&version)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_after, 1);

        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", table_name))
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM _premix_migrations WHERE version = $1")
            .bind(&version)
            .execute(&pool)
            .await;
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn postgres_migrator_rolls_back_on_error() {
        let Some(pool) = pg_pool_or_skip().await else {
            return;
        };
        let migrator = Migrator::new(pool.clone());

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let version = format!("20260102{:020}", suffix);
        let table_name = format!("premix_mig_bad_{}", suffix);
        let migrations = vec![Migration {
            version: version.clone(),
            name: "bad_sql".to_string(),
            up_sql: format!(
                "CREATE TABLE {} (id SERIAL PRIMARY KEY); INVALID SQL",
                table_name
            ),
            down_sql: format!("DROP TABLE {};", table_name),
        }];

        let err = migrator.run(migrations).await.unwrap_err();
        assert!(err.to_string().contains("syntax"));

        let table_exists: Option<String> =
            sqlx::query_scalar("SELECT to_regclass('_premix_migrations')::text")
                .fetch_one(&pool)
                .await
                .unwrap();
        if table_exists.is_some() {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations WHERE version = $1")
                    .bind(&version)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(count, 0);
        }

        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", table_name))
            .execute(&pool)
            .await;
    }

    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn postgres_migrator_rolls_back_last() {
        let Some(pool) = pg_pool_or_skip().await else {
            return;
        };
        let migrator = Migrator::new(pool.clone());

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let version_a = format!("20260103{:020}", suffix);
        let version_b = format!("20260104{:020}", suffix);
        let table_a = format!("premix_mig_a_{}", suffix);
        let table_b = format!("premix_mig_b_{}", suffix);
        let migrations = vec![
            Migration {
                version: version_a.clone(),
                name: "create_a".to_string(),
                up_sql: format!("CREATE TABLE {} (id SERIAL PRIMARY KEY);", table_a),
                down_sql: format!("DROP TABLE {};", table_a),
            },
            Migration {
                version: version_b.clone(),
                name: "create_b".to_string(),
                up_sql: format!("CREATE TABLE {} (id SERIAL PRIMARY KEY);", table_b),
                down_sql: format!("DROP TABLE {};", table_b),
            },
        ];

        migrator.run(migrations.clone()).await.unwrap();
        let rolled_back = migrator.rollback_last(migrations).await.unwrap();
        assert!(rolled_back);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(count >= 1);

        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", table_a))
            .execute(&pool)
            .await;
        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", table_b))
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM _premix_migrations WHERE version = $1 OR version = $2")
            .bind(&version_a)
            .bind(&version_b)
            .execute(&pool)
            .await;
    }
}
