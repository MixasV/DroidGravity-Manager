mod cli;
mod config;
mod oauth;
mod proxy;
mod daemon;
mod factory;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "drovity")]
#[command(about = "Google Gemini API proxy for Factory Droid", long_about = None)]
struct Cli {
    /// Enable detailed logging to ~/.drovity/proxy.log
    #[arg(long)]
    log: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive menu
    Menu,
    /// Start proxy server in foreground
    Start,
    /// Stop background proxy server
    Stop,
    /// Check proxy server status
    Status,
    /// Run proxy in background (daemon mode)
    Hide,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging only if --log flag is provided
    if cli.log {
        setup_logging()?;
    }

    match cli.command {
        Some(Commands::Menu) | None => {
            // Default: show interactive menu
            cli::menu::show_main_menu().await?;
        }
        Some(Commands::Start) => {
            // Start proxy in foreground
            daemon::start_foreground().await?;
        }
        Some(Commands::Stop) => {
            // Stop background proxy
            daemon::stop().await?;
        }
        Some(Commands::Status) => {
            // Show status
            daemon::status().await?;
        }
        Some(Commands::Hide) => {
            // Start in background
            daemon::start_background().await?;
        }
    }

    Ok(())
}

fn setup_logging() -> Result<()> {
    use std::fs::OpenOptions;
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    
    // Get log file path
    let config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".drovity");
    
    std::fs::create_dir_all(&config_dir)?;
    let log_file = config_dir.join("proxy.log");
    
    // Open log file in append mode
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;
    
    // Setup tracing subscriber with file output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with_writer(file.with_max_level(tracing::Level::INFO))
        .with_ansi(false)
        .init();
    
    Ok(())
}
