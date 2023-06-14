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
    /// Starts the Darx backend server handling data plane request.
    Server {
        /// The port to listen on.
        #[arg(short, long, default_value_t = 4001)]
        port: u16,
        #[arg(long, default_value_t = String::from("."))]
        projects_dir: String,
    },
    /// Manage database schema
    #[command(subcommand)]
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
        Commands::Server { port, projects_dir } => {
            darx_server::run_server(*port, projects_dir).await?
        }
        Commands::Dev { dir } => dev::run_dev(dir).await?,
        _ => {
            println!("other");
        }
    }
    Ok(())
}
