use anyhow::Result;
use clap::Parser;
use tracing_subscriber;

mod app;
mod data;
mod git;
mod ui;
mod utils;

use app::App;

#[derive(Parser)]
#[command(name = "scv")]
#[command(about = "Student Code Viewer - Beautiful terminal UI for managing student GitHub repositories")]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// GitHub token for API access
    #[arg(short, long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.debug { 
            tracing::Level::DEBUG 
        } else { 
            tracing::Level::INFO 
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Initialize the database
    data::database::init_db().await?;

    // Create and run the app
    let mut app = App::new(cli.github_token).await?;
    let result = app.run().await;

    // Ensure we restore the terminal before exiting
    utils::terminal::restore_terminal()?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        let app = App::new(None).await;
        assert!(app.is_ok());
    }
}