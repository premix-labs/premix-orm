use clap::{Parser, Subcommand};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("🚀 Initializing Premix project...");
            // TODO: Create default config if needed
            println!("✅ Done. You can now use #[derive(Model)] in your structs.");
        }
        Commands::Sync { database } => {
            println!("🔍 Scanning for models...");
            let db_url = database
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .unwrap_or_else(|| "sqlite:premix.db".to_string());

            println!("📡 Connecting to {}...", db_url);

            // Note: In Phase 5, this is a skeleton.
            // True 'sync' requires the CLI to know about the User's structs.
            // For now, we provide the CLI structure as requested.
            println!(
                "⚠️ CLI Sync is under construction. Please use Premix::sync::<Model>(&pool) in your code for now."
            );
        }
    }

    Ok(())
}
