use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a Darx project.
    Init,
    /// Starts the Darx development server that watches local file
    /// and use control plan API to register functions.
    Dev,
    /// Starts the Darx backend server handling data plan and control plan API.
    Server,
    /// Deploy user's backend code to Darx server.
    Deploy,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Server => darx_api_server::run_server().await?,
        _ => {
            println!("other");
        }
    }
    Ok(())
}
