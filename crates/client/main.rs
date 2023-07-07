mod dev;

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
    /// Starts the Darx development server that watches local files.
    ///
    /// This command will:
    /// 1. Guide the user to login into the Darx platform and save the
    /// api key in [`~/.darx/config.json`] file.
    /// 2. Create a new project with dev and production environments,
    /// create a directory [`darx_server`], and save environment's
    /// deployment url in [`darx_server/darx.json`] file.
    /// 3. Creates a [`functions`] directory and watch for change.
    Dev {
        /// The project's working directory.
        #[arg(short, long, default_value_t = String::from("."))]
        dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Dev { dir } => dev::run_dev(dir).await?,
    }
    Ok(())
}