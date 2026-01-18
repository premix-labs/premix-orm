use std::{fs, io::Write, path::Path, str::FromStr};

use chrono::Utc;
use clap::{Parser, Subcommand};
use premix_core::{Migration, Migrator};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "premix")]
#[command(about = "Premix ORM CLI - The Zero-Overhead Developer Experience", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Premix project
    Init,
    /// Synchronize database schema with local models
    Sync {
        #[arg(short, long)]
        database: Option<String>,
    },
    /// Manage database migrations
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
}

#[derive(Subcommand)]
enum MigrateAction {
    /// Create a new migration file
    Create {
        /// Name of the migration (e.g. create_users)
        name: String,
    },
    /// Apply pending migrations
    Up {
        #[arg(short, long)]
        database: Option<String>,
    },
    /// Revert the last migration
    Down,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!(">> Initializing Premix project...");
            println!("[OK] Done. You can now use #[derive(Model)] in your structs.");
        }
        Commands::Sync { database } => {
            println!(">> Scanning for models...");
            let db_url = database
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .unwrap_or_else(|| "sqlite:premix.db".to_string());
            println!(">> Connecting to {}...", db_url);
            println!(
                "[WARN] CLI Sync is under construction. Please use Premix::sync::<Model>(&pool) in code."
            );
        }
        Commands::Migrate { action } => match action {
            MigrateAction::Create { name } => {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S");
                let filename = format!("{}_{}.sql", timestamp, name);
                let dir_path = Path::new("migrations");

                if !dir_path.exists() {
                    fs::create_dir(dir_path)?;
                    println!(">> Created 'migrations' directory.");
                }

                let file_path = dir_path.join(&filename);
                let mut file = fs::File::create(&file_path)?;

                let content = format!(
                    "-- Migration: {}\n-- Created at: {}\n\n-- up\nCREATE TABLE {} (\n    id INTEGER PRIMARY KEY,\n    -- columns\n);\n\n-- down\nDROP TABLE {};\n",
                    name,
                    Utc::now(),
                    name,
                    name
                );

                file.write_all(content.as_bytes())?;
                println!("[OK] Created migration: migrations/{}", filename);
            }
            MigrateAction::Up { database } => {
                let db_url = database
                    .or_else(|| std::env::var("DATABASE_URL").ok())
                    .unwrap_or_else(|| "sqlite:premix.db".to_string());

                println!(">> Connecting to {}...", db_url);

                // For MVP: Support SQLite only in CLI initially
                let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
                let pool = SqlitePool::connect_with(options).await?;

                let migrations = load_migrations("migrations")?;
                if migrations.is_empty() {
                    println!("[INFO] No migrations found.");
                    return Ok(());
                }

                let migrator = Migrator::new(pool);
                migrator.run(migrations).await?;
                println!("[OK] Migrations up to date.");
            }
            MigrateAction::Down => {
                println!("Start Reverting... (Not implemented yet)");
            }
        },
    }

    Ok(())
}

fn load_migrations(path: &str) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
    let mut migrations = Vec::new();
    let dir = Path::new(path);

    if !dir.exists() {
        return Ok(migrations);
    }

    for entry in WalkDir::new(dir).min_depth(1).max_depth(1) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            // Format: YYYYMMDDHHMMSS_name.sql
            let parts: Vec<&str> = filename.splitn(2, '_').collect();
            if parts.len() != 2 {
                continue;
            }

            let version = parts[0].to_string();
            let name = parts[1].replace(".sql", "");

            let content = fs::read_to_string(path)?;
            let up_marker = "-- up";
            let down_marker = "-- down";

            let up_start = content.find(up_marker).unwrap_or(0);
            let down_start = content.find(down_marker).unwrap_or(content.len());

            let up_sql = if up_start < down_start {
                content[up_start + up_marker.len()..down_start]
                    .trim()
                    .to_string()
            } else {
                content[up_start + up_marker.len()..].trim().to_string()
            };

            let down_sql = if down_start < content.len() {
                content[down_start + down_marker.len()..].trim().to_string()
            } else {
                String::new()
            };

            migrations.push(Migration {
                version,
                name,
                up_sql,
                down_sql,
            });
        }
    }

    // Sort by version
    migrations.sort_by(|a, b| a.version.cmp(&b.version));

    Ok(migrations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    fn make_temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("premix_cli_test_{}", nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn load_migrations_empty_dir() {
        let dir = make_temp_dir();
        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert!(migrations.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_migrations_parses_and_sorts() {
        let dir = make_temp_dir();
        let file_a = dir.join("20260101000000_create_users.sql");
        let file_b = dir.join("20260102000000_create_posts.sql");
        let file_bad = dir.join("badname.sql");

        fs::write(
            &file_a,
            "-- up\nCREATE TABLE users (id INTEGER PRIMARY KEY);\n-- down\nDROP TABLE users;\n",
        )
        .unwrap();
        fs::write(
            &file_b,
            "-- up\nCREATE TABLE posts (id INTEGER PRIMARY KEY);\n-- down\nDROP TABLE posts;\n",
        )
        .unwrap();
        fs::write(&file_bad, "-- up\nSELECT 1;\n").unwrap();

        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, "20260101000000");
        assert_eq!(migrations[0].name, "create_users");
        assert!(migrations[0].up_sql.contains("CREATE TABLE users"));
        assert!(migrations[0].down_sql.contains("DROP TABLE users"));

        assert_eq!(migrations[1].version, "20260102000000");
        assert_eq!(migrations[1].name, "create_posts");
        assert!(migrations[1].up_sql.contains("CREATE TABLE posts"));
        assert!(migrations[1].down_sql.contains("DROP TABLE posts"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_migrations_handles_missing_markers() {
        let dir = make_temp_dir();
        let file = dir.join("20260103000000_partial.sql");

        fs::write(&file, "CREATE TABLE items (id INTEGER PRIMARY KEY);").unwrap();

        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, "20260103000000");
        assert_eq!(migrations[0].name, "partial");
        assert!(!migrations[0].up_sql.is_empty());
        assert!(migrations[0].down_sql.is_empty());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cli_parses_init() {
        let cli = Cli::try_parse_from(["premix", "init"]).unwrap();
        match cli.command {
            Commands::Init => {}
            _ => panic!("expected init command"),
        }
    }

    #[test]
    fn cli_parses_sync_with_db() {
        let cli = Cli::try_parse_from(["premix", "sync", "--database", "sqlite:dev.db"]).unwrap();
        match cli.command {
            Commands::Sync { database } => {
                assert_eq!(database.as_deref(), Some("sqlite:dev.db"));
            }
            _ => panic!("expected sync command"),
        }
    }

    #[test]
    fn cli_parses_sync_without_db() {
        let cli = Cli::try_parse_from(["premix", "sync"]).unwrap();
        match cli.command {
            Commands::Sync { database } => {
                assert!(database.is_none());
            }
            _ => panic!("expected sync command"),
        }
    }

    #[test]
    fn cli_parses_migrate_create() {
        let cli = Cli::try_parse_from(["premix", "migrate", "create", "create_users"]).unwrap();
        match cli.command {
            Commands::Migrate { action } => match action {
                MigrateAction::Create { name } => assert_eq!(name, "create_users"),
                _ => panic!("expected migrate create"),
            },
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn cli_parses_migrate_up_with_db() {
        let cli = Cli::try_parse_from([
            "premix",
            "migrate",
            "up",
            "--database",
            "sqlite:premix.db",
        ])
        .unwrap();
        match cli.command {
            Commands::Migrate { action } => match action {
                MigrateAction::Up { database } => {
                    assert_eq!(database.as_deref(), Some("sqlite:premix.db"));
                }
                _ => panic!("expected migrate up"),
            },
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn cli_parses_migrate_down() {
        let cli = Cli::try_parse_from(["premix", "migrate", "down"]).unwrap();
        match cli.command {
            Commands::Migrate { action } => match action {
                MigrateAction::Down => {}
                _ => panic!("expected migrate down"),
            },
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn load_migrations_ignores_non_sql_files() {
        let dir = make_temp_dir();
        let file_txt = dir.join("20260104000000_note.txt");
        fs::write(&file_txt, "hello").unwrap();

        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert!(migrations.is_empty());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_migrations_missing_dir_returns_empty() {
        let dir = make_temp_dir();
        fs::remove_dir_all(&dir).unwrap();
        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert!(migrations.is_empty());
    }

    #[test]
    fn load_migrations_parses_down_only() {
        let dir = make_temp_dir();
        let file = dir.join("20260105000000_down_only.sql");
        fs::write(&file, "-- down\nDROP TABLE items;\n").unwrap();

        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].name, "down_only");
        assert!(migrations[0].down_sql.contains("DROP TABLE items"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_migrations_up_after_down_still_parses() {
        let dir = make_temp_dir();
        let file = dir.join("20260106000000_reversed.sql");
        fs::write(
            &file,
            "-- down\nDROP TABLE items;\n-- up\nCREATE TABLE items (id INTEGER PRIMARY KEY);\n",
        )
        .unwrap();

        let migrations = load_migrations(dir.to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert!(migrations[0].up_sql.contains("CREATE TABLE"));
        assert!(migrations[0].down_sql.contains("DROP TABLE"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cli_parse_requires_subcommand() {
        let result = Cli::try_parse_from(["premix"]);
        assert!(result.is_err());
    }
}
