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
    /// Starts the Darx development server that watches local files.
    Dev,
    /// Starts the Darx backend server handling data plane request.
    Server,
    #[command(subcommand)]
    /// Manage database schema
    Schema(Schema),
    /// Manage API
    #[command(subcommand)]
    Api(Api),
}

#[derive(Subcommand)]
enum Schema {
    /// Deploy the schema migrations to a target project.
    Deploy,
    /// Rollback the schema migrations.
    Rollback,
}

#[derive(Subcommand)]
enum Api {
    /// Preview the API for a target project.
    Preview,
    /// Deploy the API to a target project.
    Deploy,
    /// Rollback the API deployment in a target project.
    Rollback,
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
