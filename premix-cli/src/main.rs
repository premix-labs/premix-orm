use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::Duration,
};

use chrono::Utc;
use clap::{Parser, Subcommand};
#[cfg(feature = "sqlite")]
use premix_core::schema;
use premix_core::schema::SchemaTable;
use premix_core::{Migration, Migrator};
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;
#[cfg(feature = "postgres")]
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "sqlite")]
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use walkdir::WalkDir;

mod source_scan;
use source_scan::{DbKind, scan_models_schema};

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
        /// Print what would happen without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Diff or generate migrations from local models
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    /// Manage database migrations
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
    /// Generate Rust models from an existing database
    Scaffold {
        #[arg(short, long)]
        database: Option<String>,
        /// Limit to a single table
        #[arg(short, long)]
        table: Option<String>,
        /// Output path for generated Rust structs
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SchemaAction {
    /// Diff database schema against local models
    Diff {
        #[arg(short, long)]
        database: Option<String>,
    },
    /// Generate a migration from schema diff
    Migrate {
        #[arg(short, long)]
        database: Option<String>,
        /// Output path for the migration file
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Print generated SQL without writing a migration file
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
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
        /// Print pending migrations without applying
        #[arg(long)]
        dry_run: bool,
    },
    /// Revert the last migration
    Down {
        #[arg(short, long)]
        database: Option<String>,
        /// Print the migration that would be reverted
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
    },
}

enum SchemaMigrateOutcome {
    Created(PathBuf),
    DryRun,
    NoChanges,
    Aborted,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    run_cli(cli).await
}

async fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cli.command {
        Commands::Init => {
            println!(">> Initializing Premix project...");
            init_project()?;
            println!("[OK] Done. You can now use #[derive(Model)] in your structs.");
        }
        Commands::Sync { database, dry_run } => {
            println!(">> Scanning for models...");
            let db_url = resolve_db_url(database);
            let db_kind = DbKind::from_url(&db_url);
            let expected = scan_models_schema(Path::new("src"), db_kind)?;
            if expected.is_empty() {
                println!(
                    "[WARN] No models with #[derive(Model)] found under src/. Nothing to sync."
                );
                return Ok(());
            }
            println!(">> Connecting to {}...", db_url);
            if dry_run {
                print_sync_dry_run(&expected, db_kind);
                return Ok(());
            }
            run_sync_scanned(&db_url, db_kind, &expected).await?;
            println!("[OK] Sync completed.");
        }
        Commands::Schema { action } => match action {
            SchemaAction::Diff { database } => {
                let db_url = resolve_db_url(database);
                let db_kind = DbKind::from_url(&db_url);
                let expected = scan_models_schema(Path::new("src"), db_kind)?;
                if expected.is_empty() {
                    println!(
                        "[WARN] No models with #[derive(Model)] found under src/. Nothing to diff."
                    );
                    return Ok(());
                }
                println!(">> Connecting to {}...", db_url);
                let output = run_schema_diff(&db_url, &expected).await?;
                print!("{}", output);
                println!("[OK] Schema diff complete.");
            }
            SchemaAction::Migrate {
                database,
                out,
                dry_run,
                yes,
            } => {
                let db_url = resolve_db_url(database);
                let db_kind = DbKind::from_url(&db_url);
                let expected = scan_models_schema(Path::new("src"), db_kind)?;
                if expected.is_empty() {
                    println!(
                        "[WARN] No models with #[derive(Model)] found under src/. Nothing to migrate."
                    );
                    return Ok(());
                }
                println!(">> Connecting to {}...", db_url);
                match run_schema_migrate(&db_url, &expected, out, dry_run, yes).await? {
                    SchemaMigrateOutcome::Created(path) => {
                        println!("[OK] Schema migration created: {}", path.to_string_lossy());
                    }
                    SchemaMigrateOutcome::DryRun => {
                        println!("[OK] Schema migration dry run complete.");
                    }
                    SchemaMigrateOutcome::NoChanges => {
                        println!("[INFO] Schema diff: no changes.");
                    }
                    SchemaMigrateOutcome::Aborted => {
                        println!("[INFO] Schema migration aborted.");
                    }
                }
            }
        },
        Commands::Migrate { action } => match action {
            MigrateAction::Create { name } => {
                let dir_path = Path::new("migrations");
                let file_path = create_migration_file(&name, dir_path)?;
                let filename = file_path.file_name().unwrap().to_string_lossy();
                println!("[OK] Created migration: migrations/{}", filename);
            }
            MigrateAction::Up { database, dry_run } => {
                let db_url = resolve_db_url(database);
                println!(">> Connecting to {}...", db_url);

                if dry_run {
                    dry_run_migrations_up(&db_url, Path::new("migrations")).await?;
                    return Ok(());
                }

                if !run_migrations_up(&db_url, Path::new("migrations")).await? {
                    println!("[INFO] No migrations found.");
                    return Ok(());
                }

                println!("[OK] Migrations up to date.");
            }
            MigrateAction::Down {
                database,
                dry_run,
                yes,
            } => {
                let db_url = resolve_db_url(database);
                println!(">> Connecting to {}...", db_url);

                if dry_run {
                    dry_run_migrations_down(&db_url, Path::new("migrations")).await?;
                    return Ok(());
                }

                if db_url.starts_with("sqlite:") || db_url.starts_with("sqlite://") {
                    println!(
                        "[WARN] SQLite down migrations may require table recreation and can cause data loss."
                    );
                }
                if !yes && !confirm_action("Proceed with migrate down?")? {
                    println!("[INFO] Migration rollback cancelled.");
                    return Ok(());
                }

                if !run_migrations_down(&db_url, Path::new("migrations")).await? {
                    println!("[INFO] No migrations to roll back.");
                    return Ok(());
                }

                println!("[OK] Last migration reverted.");
            }
        },
        Commands::Scaffold {
            database,
            table,
            out,
        } => {
            let db_url = resolve_db_url(database);
            println!(">> Connecting to {}...", db_url);
            let output = run_scaffold(&db_url, table.as_deref()).await?;
            if let Some(out_path) = out {
                let mut file = fs::File::create(&out_path)?;
                file.write_all(output.as_bytes())?;
                println!("[OK] Scaffold written to {}", out_path.to_string_lossy());
            } else {
                println!("{}", output);
            }
        }
    }

    Ok(())
}

fn init_project() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[INFO] premix init no longer creates premix-sync/premix-schema binaries.");
    println!("[INFO] The CLI scans src/ for #[derive(Model)] automatically.");
    Ok(())
}

fn confirm_action(prompt: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim().to_ascii_lowercase();
    Ok(matches!(value.as_str(), "y" | "yes"))
}

async fn dry_run_migrations_up(
    db_url: &str,
    migrations_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let migrations_dir = migrations_dir.to_string_lossy();
    let migrations = load_migrations(&migrations_dir)?;
    if migrations.is_empty() {
        println!("[INFO] No migrations found.");
        return Ok(());
    }

    let pending = pending_migrations(db_url, &migrations).await?;
    if pending.is_empty() {
        println!("[INFO] No pending migrations.");
        return Ok(());
    }

    println!("[INFO] Pending migrations:");
    for migration in pending {
        println!("  - {} {}", migration.version, migration.name);
    }
    Ok(())
}

async fn dry_run_migrations_down(
    db_url: &str,
    migrations_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let migrations_dir = migrations_dir.to_string_lossy();
    let migrations = load_migrations(&migrations_dir)?;
    if migrations.is_empty() {
        println!("[INFO] No migrations found.");
        return Ok(());
    }

    let last = last_applied_migration(db_url, &migrations).await?;
    match last {
        Some(migration) => {
            println!(
                "[INFO] Would roll back: {} {}",
                migration.version, migration.name
            );
        }
        None => {
            println!("[INFO] No migrations to roll back.");
        }
    }
    Ok(())
}

async fn pending_migrations(
    db_url: &str,
    migrations: &[Migration],
) -> Result<Vec<Migration>, Box<dyn std::error::Error + Send + Sync>> {
    if migrations.is_empty() {
        return Ok(Vec::new());
    }

    let applied = if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            applied_versions_postgres(db_url).await?
        }
        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    } else {
        #[cfg(feature = "sqlite")]
        {
            applied_versions_sqlite(db_url).await?
        }
        #[cfg(not(feature = "sqlite"))]
        {
            return Err(
                "SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into(),
            );
        }
    };

    let applied_set: HashSet<String> = applied.into_iter().collect();
    Ok(migrations
        .iter()
        .filter(|m| !applied_set.contains(&m.version))
        .cloned()
        .collect())
}

async fn last_applied_migration(
    db_url: &str,
    migrations: &[Migration],
) -> Result<Option<Migration>, Box<dyn std::error::Error + Send + Sync>> {
    if migrations.is_empty() {
        return Ok(None);
    }

    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            return last_applied_postgres(db_url, migrations).await;
        }
        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    }

    #[cfg(feature = "sqlite")]
    {
        last_applied_sqlite(db_url, migrations).await
    }

    #[cfg(not(feature = "sqlite"))]
    {
        Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
    }
}

#[cfg(feature = "sqlite")]
async fn applied_versions_sqlite(
    db_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
    let table: Option<String> = with_sqlite_retry("check migrations table", || async {
        sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='_premix_migrations'",
        )
        .fetch_optional(&pool)
        .await
    })
    .await?;
    if table.is_none() {
        return Ok(Vec::new());
    }

    let versions: Vec<String> = with_sqlite_retry("load migrations versions", || async {
        sqlx::query_scalar("SELECT version FROM _premix_migrations ORDER BY version ASC")
            .fetch_all(&pool)
            .await
    })
    .await?;
    Ok(versions)
}

#[cfg(feature = "postgres")]
async fn applied_versions_postgres(
    db_url: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = PgPoolOptions::new().connect(db_url).await?;
    let table_exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('_premix_migrations')::text")
            .fetch_one(&pool)
            .await?;
    if table_exists.is_none() {
        return Ok(Vec::new());
    }

    let versions: Vec<String> =
        sqlx::query_scalar("SELECT version FROM _premix_migrations ORDER BY version ASC")
            .fetch_all(&pool)
            .await?;
    Ok(versions)
}

#[cfg(feature = "sqlite")]
async fn last_applied_sqlite(
    db_url: &str,
    migrations: &[Migration],
) -> Result<Option<Migration>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
    let table: Option<String> = with_sqlite_retry("check migrations table", || async {
        sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='_premix_migrations'",
        )
        .fetch_optional(&pool)
        .await
    })
    .await?;
    if table.is_none() {
        return Ok(None);
    }

    let versions: Vec<String> = migrations.iter().map(|m| m.version.clone()).collect();
    let placeholders = vec!["?"; versions.len()].join(", ");
    let sql = format!(
        "SELECT version FROM _premix_migrations WHERE version IN ({}) ORDER BY version DESC LIMIT 1",
        placeholders
    );
    let last = with_sqlite_retry("select last migration", || async {
        let mut query = sqlx::query_scalar::<sqlx::Sqlite, String>(&sql);
        for version in &versions {
            query = query.bind(version);
        }
        query.fetch_optional(&pool).await
    })
    .await?;
    Ok(last.and_then(|version| migrations.iter().find(|m| m.version == version).cloned()))
}

#[cfg(feature = "postgres")]
async fn last_applied_postgres(
    db_url: &str,
    migrations: &[Migration],
) -> Result<Option<Migration>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = PgPoolOptions::new().connect(db_url).await?;
    let table_exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('_premix_migrations')::text")
            .fetch_one(&pool)
            .await?;
    if table_exists.is_none() {
        return Ok(None);
    }

    let versions: Vec<String> = migrations.iter().map(|m| m.version.clone()).collect();
    let last = sqlx::query_scalar::<sqlx::Postgres, String>(
        "SELECT version FROM _premix_migrations WHERE version = ANY($1) ORDER BY version DESC LIMIT 1",
    )
    .bind(&versions)
    .fetch_optional(&pool)
    .await?;
    Ok(last.and_then(|version| migrations.iter().find(|m| m.version == version).cloned()))
}

fn to_pascal_case(name: &str) -> String {
    let mut out = String::new();
    for part in name.split('_').filter(|p| !p.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push_str(&first.to_uppercase().to_string());
            out.push_str(&chars.as_str().to_lowercase());
        }
    }
    out
}

fn singularize(name: &str) -> &str {
    if name.ends_with('s') && name.len() > 1 {
        &name[..name.len() - 1]
    } else {
        name
    }
}

fn rust_type_for_sql(sql_type: &str, nullable: bool) -> String {
    let sql_type = sql_type.to_ascii_lowercase();
    let base = if sql_type.contains("int") && !sql_type.contains("big") {
        "i32".to_string()
    } else if sql_type.contains("bigint") || sql_type.contains("int8") {
        "i64".to_string()
    } else if sql_type.contains("bool") {
        "bool".to_string()
    } else if sql_type.contains("real")
        || sql_type.contains("double")
        || sql_type.contains("numeric")
        || sql_type.contains("decimal")
        || sql_type.contains("float")
    {
        "f64".to_string()
    } else {
        "String".to_string()
    };

    if nullable && base != "String" {
        format!("Option<{}>", base)
    } else if nullable && base == "String" {
        "Option<String>".to_string()
    } else {
        base
    }
}

#[cfg(feature = "postgres")]
fn postgres_base_rust_type(data_type: &str) -> String {
    let t = data_type.to_ascii_lowercase();
    if t.contains("int2") || t.contains("int4") || t == "integer" || t == "smallint" {
        "i32".to_string()
    } else if t.contains("int8") || t == "bigint" {
        "i64".to_string()
    } else if t.contains("bool") {
        "bool".to_string()
    } else if t.contains("double")
        || t.contains("float8")
        || t.contains("real")
        || t.contains("float4")
        || t.contains("numeric")
        || t.contains("decimal")
    {
        "f64".to_string()
    } else if t.contains("bytea") {
        "Vec<u8>".to_string()
    } else {
        "String".to_string()
    }
}

#[cfg(feature = "postgres")]
fn rust_type_for_postgres(data_type: &str, udt_name: &str, nullable: bool) -> String {
    let data_type = data_type.to_ascii_lowercase();
    let udt_name = udt_name.to_ascii_lowercase();

    let (base, is_array) = if data_type == "array" {
        let base = udt_name.trim_start_matches('_');
        (postgres_base_rust_type(base), true)
    } else if data_type == "user-defined" {
        (postgres_base_rust_type(&udt_name), false)
    } else {
        (postgres_base_rust_type(&data_type), false)
    };

    let rust_type = if is_array {
        format!("Vec<{}>", base)
    } else {
        base
    };

    if nullable {
        format!("Option<{}>", rust_type)
    } else {
        rust_type
    }
}

async fn run_scaffold(
    db_url: &str,
    table: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            return scaffold_postgres(&pool, table).await;
        }
        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    }

    #[cfg(feature = "sqlite")]
    {
        let pool =
            with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
        return scaffold_sqlite(&pool, table).await;
    }

    #[cfg(not(feature = "sqlite"))]
    {
        Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
    }
}

#[cfg(feature = "sqlite")]
async fn scaffold_sqlite(
    pool: &SqlitePool,
    table: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let tables = schema::introspect_sqlite_schema(pool).await?;
    let tables = tables
        .into_iter()
        .filter(|t| table.map(|name| name == t.name).unwrap_or(true))
        .collect::<Vec<_>>();

    let mut out = String::from("use premix_orm::prelude::*;\n\n");
    for table in tables {
        let struct_name = to_pascal_case(singularize(&table.name));
        out.push_str("#[derive(Model, Debug)]\n");
        out.push_str(&format!("struct {} {{\n", struct_name));
        for col in table.columns {
            let rust_type = rust_type_for_sql(&col.sql_type, col.nullable);
            out.push_str(&format!("    {}: {},\n", col.name, rust_type));
        }
        out.push_str("}\n\n");
    }
    Ok(out)
}

#[cfg(feature = "postgres")]
async fn scaffold_postgres(
    pool: &sqlx::PgPool,
    table: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let tables: Vec<String> = if let Some(name) = table {
        vec![name.to_string()]
    } else {
        sqlx::query_scalar(
            "SELECT table_name FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE' ORDER BY table_name",
        )
        .fetch_all(pool)
        .await?
    };

    let mut out = String::from("use premix_orm::prelude::*;\n\n");
    for table_name in tables {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT column_name, data_type, udt_name, is_nullable FROM information_schema.columns WHERE table_schema='public' AND table_name=$1 ORDER BY ordinal_position",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;

        if rows.is_empty() {
            continue;
        }

        let struct_name = to_pascal_case(singularize(&table_name));
        out.push_str("#[derive(Model, Debug)]\n");
        out.push_str(&format!("struct {} {{\n", struct_name));
        for (col_name, data_type, udt_name, is_nullable) in rows {
            let nullable = is_nullable.eq_ignore_ascii_case("YES");
            let rust_type = rust_type_for_postgres(&data_type, &udt_name, nullable);
            out.push_str(&format!("    {}: {},\n", col_name, rust_type));
        }
        out.push_str("}\n\n");
    }
    Ok(out)
}

fn resolve_db_url(database: Option<String>) -> String {
    database
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| "sqlite:premix.db".to_string())
}

#[cfg(feature = "sqlite")]
fn sqlite_connect_options(db_url: &str) -> Result<SqliteConnectOptions, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .statement_cache_capacity(200)
        .busy_timeout(Duration::from_secs(5));
    Ok(options)
}

#[cfg(feature = "sqlite")]
async fn sqlite_pool(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect_with(sqlite_connect_options(db_url)?).await
}

#[cfg(feature = "sqlite")]
fn is_retryable_sqlite_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Io(io) => io.raw_os_error() == Some(32),
        sqlx::Error::Database(db) => {
            let msg = db.message().to_ascii_lowercase();
            msg.contains("database is locked")
                || msg.contains("database is busy")
                || msg.contains("sqlite_busy")
                || msg.contains("lock")
        }
        _ => false,
    }
}

#[cfg(feature = "sqlite")]
async fn with_sqlite_retry<T, F, Fut>(label: &str, mut f: F) -> Result<T, sqlx::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    let mut attempt = 0usize;
    let mut delay = Duration::from_millis(50);
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) if is_retryable_sqlite_error(&err) && attempt < 6 => {
                attempt += 1;
                eprintln!(
                    "[WARN] SQLite busy/locked during {}, retrying (attempt {}).",
                    label, attempt
                );
                if attempt == 1 {
                    try_release_rustc_handles();
                }
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_millis(800));
            }
            Err(err) => return Err(err),
        }
    }
}

fn try_release_rustc_handles() {
    let enabled = std::env::var("PREMIX_SIGNAL_RUSTC").ok();
    if enabled.as_deref() != Some("1") {
        return;
    }

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/IM", "rustc.exe", "/T"])
            .status();
        eprintln!("[INFO] Sent taskkill to rustc.exe (PREMIX_SIGNAL_RUSTC=1).");
    }

    #[cfg(not(windows))]
    {
        let _ = Command::new("pkill").args(["-TERM", "rustc"]).status();
        eprintln!("[INFO] Sent SIGTERM to rustc (PREMIX_SIGNAL_RUSTC=1).");
    }
}

fn create_migration_file(
    name: &str,
    dir_path: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    if !dir_path.exists() {
        fs::create_dir(dir_path)?;
        println!(">> Created 'migrations' directory.");
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_{}.sql", timestamp, name);
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
    Ok(file_path)
}

async fn run_migrations_up(
    db_url: &str,
    migrations_dir: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let migrations_dir = migrations_dir.to_string_lossy();
    let migrations = load_migrations(&migrations_dir)?;
    if migrations.is_empty() {
        return Ok(false);
    }

    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            let migrator = Migrator::new(pool);
            migrator.run(migrations).await?;
            return Ok(true);
        }

        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    }

    #[cfg(feature = "sqlite")]
    {
        let pool =
            with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
        let migrator = Migrator::new(pool);
        migrator.run(migrations).await?;
        Ok(true)
    }

    #[cfg(not(feature = "sqlite"))]
    {
        Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
    }
}

async fn run_migrations_down(
    db_url: &str,
    migrations_dir: &Path,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let migrations_dir = migrations_dir.to_string_lossy();
    let migrations = load_migrations(&migrations_dir)?;
    if migrations.is_empty() {
        return Ok(false);
    }

    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            let migrator = Migrator::new(pool);
            return migrator.rollback_last(migrations).await;
        }

        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    }

    #[cfg(feature = "sqlite")]
    {
        let pool =
            with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
        let migrator = Migrator::new(pool);
        return migrator.rollback_last(migrations).await;
    }

    #[cfg(not(feature = "sqlite"))]
    {
        Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
    }
}

fn load_migrations(path: &str) -> Result<Vec<Migration>, Box<dyn std::error::Error + Send + Sync>> {
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

fn print_sync_dry_run(expected: &[SchemaTable], db_kind: DbKind) {
    println!("[INFO] Dry run: would create {} tables.", expected.len());
    for table in expected {
        let sql = create_sql_for_table(table, db_kind);
        println!("-- {}", table.name);
        println!("{}", sql);
    }
}

async fn run_sync_scanned(
    db_url: &str,
    db_kind: DbKind,
    expected: &[SchemaTable],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if db_url.starts_with("mysql://") {
        return Err(
            "MySQL support is not enabled. Rebuild premix-cli with --features mysql.".into(),
        );
    }
    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            for table in expected {
                let sql = create_sql_for_table(table, db_kind);
                sqlx::query(&sql).execute(&pool).await?;
            }
            Ok(())
        }
        #[cfg(not(feature = "postgres"))]
        {
            Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            )
        }
    } else {
        #[cfg(feature = "sqlite")]
        {
            let pool =
                with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
            for table in expected {
                let sql = create_sql_for_table(table, db_kind);
                with_sqlite_retry("sync create table", || async {
                    sqlx::query(&sql).execute(&pool).await.map(|_| ())
                })
                .await?;
            }
            Ok(())
        }

        #[cfg(not(feature = "sqlite"))]
        {
            Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
        }
    }
}

async fn run_schema_diff(
    db_url: &str,
    expected: &[SchemaTable],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if db_url.starts_with("mysql://") {
        return Err(
            "MySQL support is not enabled. Rebuild premix-cli with --features mysql.".into(),
        );
    }
    if db_url.starts_with("postgres://") || db_url.starts_with("postgresql://") {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            let diff = schema::diff_postgres_schema(&pool, expected).await?;
            Ok(schema::format_schema_diff_summary(&diff))
        }
        #[cfg(not(feature = "postgres"))]
        {
            Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            )
        }
    } else {
        #[cfg(feature = "sqlite")]
        {
            let pool =
                with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
            let diff = with_sqlite_retry("schema diff", || async {
                schema::diff_sqlite_schema(&pool, expected).await
            })
            .await?;
            Ok(schema::format_schema_diff_summary(&diff))
        }

        #[cfg(not(feature = "sqlite"))]
        {
            Err("SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into())
        }
    }
}

async fn run_schema_migrate(
    db_url: &str,
    expected: &[SchemaTable],
    out: Option<PathBuf>,
    dry_run: bool,
    yes: bool,
) -> Result<SchemaMigrateOutcome, Box<dyn std::error::Error + Send + Sync>> {
    if db_url.starts_with("mysql://") {
        return Err(
            "MySQL support is not enabled. Rebuild premix-cli with --features mysql.".into(),
        );
    }
    let (diff, sql_statements) = if db_url.starts_with("postgres://")
        || db_url.starts_with("postgresql://")
    {
        #[cfg(feature = "postgres")]
        {
            let pool = PgPoolOptions::new().connect(db_url).await?;
            let diff = schema::diff_postgres_schema(&pool, expected).await?;
            let sql = schema::postgres_migration_sql(expected, &diff);
            (diff, sql)
        }
        #[cfg(not(feature = "postgres"))]
        {
            return Err(
                "Postgres support is not enabled. Rebuild premix-cli with --features postgres."
                    .into(),
            );
        }
    } else {
        #[cfg(feature = "sqlite")]
        {
            let pool =
                with_sqlite_retry("sqlite connect", || async { sqlite_pool(db_url).await }).await?;
            let diff = with_sqlite_retry("schema diff", || async {
                schema::diff_sqlite_schema(&pool, expected).await
            })
            .await?;
            let sql = schema::sqlite_migration_sql(expected, &diff);
            (diff, sql)
        }
        #[cfg(not(feature = "sqlite"))]
        {
            return Err(
                "SQLite support is not enabled. Rebuild premix-cli with --features sqlite.".into(),
            );
        }
    };

    let summary = schema::format_schema_diff_summary(&diff);
    if !summary.trim().is_empty() {
        print!("{}", summary);
        println!();
    }

    if sql_statements.is_empty() {
        return Ok(SchemaMigrateOutcome::NoChanges);
    }

    let sql = format!("{};\n", sql_statements.join(";\n"));

    if dry_run {
        println!("{}", sql);
        return Ok(SchemaMigrateOutcome::DryRun);
    }

    if !yes && !confirm_action("Generate schema migration file?")? {
        return Ok(SchemaMigrateOutcome::Aborted);
    }

    let path = if let Some(path) = out {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        write_schema_migration_file(sql.trim(), &path)?;
        path
    } else {
        let dir_path = Path::new("migrations");
        create_schema_migration_file(sql.trim(), dir_path)?
    };

    Ok(SchemaMigrateOutcome::Created(path))
}

fn create_sql_for_table(table: &SchemaTable, db_kind: DbKind) -> String {
    let mut cols = Vec::new();
    for col in &table.columns {
        if col.primary_key {
            cols.push(format!("{} {}", col.name, auto_increment_pk(db_kind)));
            continue;
        }
        let mut def = format!("{} {}", col.name, col.sql_type);
        if !col.nullable {
            def.push_str(" NOT NULL");
        }
        cols.push(def);
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table.name,
        cols.join(", ")
    )
}

fn auto_increment_pk(db_kind: DbKind) -> &'static str {
    match db_kind {
        DbKind::Sqlite => "INTEGER PRIMARY KEY",
        DbKind::Postgres => "SERIAL PRIMARY KEY",
        DbKind::Mysql => "INTEGER AUTO_INCREMENT PRIMARY KEY",
    }
}

fn create_schema_migration_file(
    sql: &str,
    dir_path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    if !dir_path.exists() {
        fs::create_dir(dir_path)?;
        println!(">> Created 'migrations' directory.");
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let filename = format!("{}_schema_diff.sql", timestamp);
    let file_path = dir_path.join(&filename);
    write_schema_migration_file(sql, &file_path)?;
    Ok(file_path)
}

fn write_schema_migration_file(
    sql: &str,
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::create(file_path)?;
    let content = format!(
        "-- Migration: schema_diff\n-- Created at: {}\n\n-- up\n{}\n\n-- down\n-- TODO: add down migration\n",
        Utc::now(),
        sql.trim()
    );
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    use sqlx::SqlitePool;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    static CWD_LOCK: Mutex<()> = Mutex::new(());
    static TEMP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn make_temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let count = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("premix_cli_test_{}_{}", nanos, count));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sqlite_test_url(root: &Path) -> String {
        let db_path = root.join("test.db");
        format!("sqlite:{}", db_path.to_string_lossy().replace('\\', "/"))
    }

    fn write_sample_model(root: &Path) {
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let content = r#"
use premix_orm::prelude::*;

#[derive(Model, Debug, Clone)]
struct User {
    id: i32,
    name: String,
}
"#;
        fs::write(src_dir.join("main.rs"), content.trim()).unwrap();
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
    fn resolve_db_url_uses_default_when_missing() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
        assert_eq!(resolve_db_url(None), "sqlite:premix.db");
    }

    #[test]
    fn resolve_db_url_prefers_env_over_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("DATABASE_URL", "sqlite:env.db");
        }
        assert_eq!(resolve_db_url(None), "sqlite:env.db");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn resolve_db_url_prefers_cli_value() {
        assert_eq!(
            resolve_db_url(Some("sqlite:cli.db".to_string())),
            "sqlite:cli.db"
        );
    }

    #[test]
    fn create_migration_file_writes_template() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        let file_path = create_migration_file("create_users", &dir).unwrap();
        let filename = file_path.file_name().unwrap().to_string_lossy();
        assert!(filename.contains("create_users"));
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("-- up"));
        assert!(content.contains("-- down"));
        assert!(content.contains("CREATE TABLE create_users"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[tokio::test]
    async fn run_migrations_up_returns_false_when_empty() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        fs::create_dir_all(&dir).unwrap();

        let db_url = sqlite_test_url(&root);
        let ran = run_migrations_up(&db_url, &dir).await.unwrap();
        assert!(!ran);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn run_migrations_up_applies_migration() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("20260107000000_create_items.sql");
        fs::write(
            &file,
            "-- up\nCREATE TABLE items (id INTEGER PRIMARY KEY);\n-- down\nDROP TABLE items;\n",
        )
        .unwrap();

        let db_url = sqlite_test_url(&root);
        let ran = run_migrations_up(&db_url, &dir).await.unwrap();
        assert!(ran);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn run_migrations_up_returns_error_on_bad_sql() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("20260108000000_bad.sql");
        fs::write(&file, "-- up\nINVALID SQL\n-- down\nDROP TABLE nope;\n").unwrap();

        let db_url = sqlite_test_url(&root);
        let err = run_migrations_up(&db_url, &dir).await.unwrap_err();
        assert!(err.to_string().contains("syntax"));

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn run_migrations_down_returns_false_when_empty() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        fs::create_dir_all(&dir).unwrap();

        let db_url = sqlite_test_url(&root);
        let ran = run_migrations_down(&db_url, &dir).await.unwrap();
        assert!(!ran);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn run_migrations_down_reverts_last() {
        let root = make_temp_dir();
        let dir = root.join("migrations");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("20260109000000_create_items.sql");
        fs::write(
            &file,
            "-- up\nCREATE TABLE items (id INTEGER PRIMARY KEY);\n-- down\nDROP TABLE items;\n",
        )
        .unwrap();

        let db_url = sqlite_test_url(&root);
        run_migrations_up(&db_url, &dir).await.unwrap();
        let rolled_back = run_migrations_down(&db_url, &dir).await.unwrap();
        assert!(rolled_back);

        let pool = SqlitePool::connect(&db_url).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _premix_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);

        let _ = fs::remove_dir_all(&root);
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
            Commands::Sync { database, dry_run } => {
                assert_eq!(database.as_deref(), Some("sqlite:dev.db"));
                assert!(!dry_run);
            }
            _ => panic!("expected sync command"),
        }
    }

    #[test]
    fn cli_parses_sync_without_db() {
        let cli = Cli::try_parse_from(["premix", "sync"]).unwrap();
        match cli.command {
            Commands::Sync { database, dry_run } => {
                assert!(database.is_none());
                assert!(!dry_run);
            }
            _ => panic!("expected sync command"),
        }
    }

    #[test]
    fn cli_parses_schema_diff() {
        let cli = Cli::try_parse_from(["premix", "schema", "diff", "--database", "sqlite:dev.db"])
            .unwrap();
        match cli.command {
            Commands::Schema { action } => match action {
                SchemaAction::Diff { database } => {
                    assert_eq!(database.as_deref(), Some("sqlite:dev.db"));
                }
                _ => panic!("expected schema diff"),
            },
            _ => panic!("expected schema command"),
        }
    }

    #[test]
    fn cli_parses_schema_migrate_with_out() {
        let cli = Cli::try_parse_from([
            "premix",
            "schema",
            "migrate",
            "--database",
            "sqlite:dev.db",
            "--out",
            "migrations/20260101000000_schema.sql",
        ])
        .unwrap();
        match cli.command {
            Commands::Schema { action } => match action {
                SchemaAction::Migrate {
                    database,
                    out,
                    dry_run,
                    yes,
                } => {
                    assert_eq!(database.as_deref(), Some("sqlite:dev.db"));
                    assert_eq!(
                        out.unwrap().to_string_lossy(),
                        "migrations/20260101000000_schema.sql"
                    );
                    assert!(!dry_run);
                    assert!(!yes);
                }
                _ => panic!("expected schema migrate"),
            },
            _ => panic!("expected schema command"),
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
        let cli =
            Cli::try_parse_from(["premix", "migrate", "up", "--database", "sqlite:premix.db"])
                .unwrap();
        match cli.command {
            Commands::Migrate { action } => match action {
                MigrateAction::Up { database, dry_run } => {
                    assert_eq!(database.as_deref(), Some("sqlite:premix.db"));
                    assert!(!dry_run);
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
                MigrateAction::Down {
                    database,
                    dry_run,
                    yes,
                } => {
                    assert!(database.is_none());
                    assert!(!dry_run);
                    assert!(!yes);
                }
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

    #[tokio::test]
    async fn cli_run_init_ok() {
        run_cli(Cli {
            command: Commands::Init,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn cli_run_sync_ok() {
        let root = make_temp_dir();
        write_sample_model(&root);

        let _lock = CWD_LOCK.lock().unwrap();
        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        let result = run_cli(Cli {
            command: Commands::Sync {
                database: Some(sqlite_test_url(&root)),
                dry_run: true,
            },
        })
        .await;

        env::set_current_dir(old_cwd).unwrap();
        result.unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn cli_run_schema_diff_ok() {
        let root = make_temp_dir();
        write_sample_model(&root);

        let _lock = CWD_LOCK.lock().unwrap();
        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        let result = run_cli(Cli {
            command: Commands::Schema {
                action: SchemaAction::Diff {
                    database: Some(sqlite_test_url(&root)),
                },
            },
        })
        .await;

        env::set_current_dir(old_cwd).unwrap();
        result.unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn cli_run_migrate_down_ok() {
        run_cli(Cli {
            command: Commands::Migrate {
                action: MigrateAction::Down {
                    database: Some("sqlite::memory:".to_string()),
                    dry_run: true,
                    yes: true,
                },
            },
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn cli_run_migrate_create_ok() {
        let root = make_temp_dir();
        fs::create_dir_all(root.join("migrations")).unwrap();

        let _lock = CWD_LOCK.lock().unwrap();
        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        let result = run_cli(Cli {
            command: Commands::Migrate {
                action: MigrateAction::Create {
                    name: "create_users".to_string(),
                },
            },
        })
        .await;

        env::set_current_dir(old_cwd).unwrap();
        result.unwrap();

        let files = fs::read_dir(root.join("migrations"))
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn cli_run_migrate_up_no_migrations_ok() {
        let root = make_temp_dir();
        fs::create_dir_all(root.join("migrations")).unwrap();

        let _lock = CWD_LOCK.lock().unwrap();
        let old_cwd = env::current_dir().unwrap();
        env::set_current_dir(&root).unwrap();

        let db_url = sqlite_test_url(&root);
        let result = run_cli(Cli {
            command: Commands::Migrate {
                action: MigrateAction::Up {
                    database: Some(db_url),
                    dry_run: false,
                },
            },
        })
        .await;

        env::set_current_dir(old_cwd).unwrap();
        result.unwrap();

        let _ = fs::remove_dir_all(&root);
    }
}
