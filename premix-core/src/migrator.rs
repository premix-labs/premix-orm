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
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

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
}
