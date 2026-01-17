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
            println!("🚀 Initializing Premix project...");
            println!("✅ Done. You can now use #[derive(Model)] in your structs.");
        }
        Commands::Sync { database } => {
            println!("🔍 Scanning for models...");
            let db_url = database
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .unwrap_or_else(|| "sqlite:premix.db".to_string());
            println!("📡 Connecting to {}...", db_url);
            println!(
                "⚠️ CLI Sync is under construction. Please use Premix::sync::<Model>(&pool) in code."
            );
        }
        Commands::Migrate { action } => match action {
            MigrateAction::Create { name } => {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S");
                let filename = format!("{}_{}.sql", timestamp, name);
                let dir_path = Path::new("migrations");

                if !dir_path.exists() {
                    fs::create_dir(dir_path)?;
                    println!("📂 Created 'migrations' directory.");
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
                println!("✅ Created migration: migrations/{}", filename);
            }
            MigrateAction::Up { database } => {
                let db_url = database
                    .or_else(|| std::env::var("DATABASE_URL").ok())
                    .unwrap_or_else(|| "sqlite:premix.db".to_string());

                println!("📡 Connecting to {}...", db_url);

                // For MVP: Support SQLite only in CLI initially
                let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
                let pool = SqlitePool::connect_with(options).await?;

                let migrations = load_migrations("migrations")?;
                if migrations.is_empty() {
                    println!("✨ No migrations found.");
                    return Ok(());
                }

                let migrator = Migrator::new(pool);
                migrator.run(migrations).await?;
                println!("✅ Migrations up to date.");
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
