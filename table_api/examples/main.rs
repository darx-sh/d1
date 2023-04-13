use crate::catalog::{init_catalog, CatalogRef};
use crate::table_api::list_entity;
use anyhow::Result;

use axum::routing::get;
use axum::Router;
use clap::{Parser, Subcommand};

use sqlx::mysql::MySqlPool;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a Darx project.
    Init,
    /// Starts a Darx dev server for local development.
    /// Dev server will watch for changes in the project and automatically register/deregister
    /// functions to Darx server through Darx' control plan.
    Dev,
    /// Starts the Darx backend server for serving requests from the frontend.
    /// Backend server also serves as a control plane API for function registration/deregistration.
    Server,
    /// Deploy the Darx application.
    Deploy,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Dev => run_dev().await?,
        _ => {
            println!("other");
        }
    }
    Ok(())
}

async fn run_dev() -> Result<()> {
    println!("run_dev");

    let pool = MySqlPool::connect("mysql://root:12345678@localhost:3306/mysql")
        .await
        .unwrap();
    init_catalog(&pool).await?;

    let app = Router::new().route(
        "/darx/api/table/:table_name",
        get(list_entity)
            .post(create_entity)
            .patch(update_entity)
            .delete(delete_entity),
    );
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

struct ServerContext {
    catalog: CatalogRef,
    pool: MySqlPool,
}

async fn create_entity() {}
async fn update_entity() {}
async fn delete_entity() {}
