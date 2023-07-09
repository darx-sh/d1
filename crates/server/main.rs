use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[derive(Debug, Parser)]
#[command(name = "darx-server")]
struct Cli {
    #[clap(
        long,
        short,
        default_value = "data_plane",
        env = "DARX_DATA_PLANE_PATH"
    )]
    data_plane_dir: PathBuf,

    #[clap(
        long,
        default_value = "127.0.0.1:3456",
        env = "DARX_DATA_PLAN_LISTEN_ADDR"
    )]
    data_plan_addr: SocketAddr,

    #[clap(long, default_value = "127.0.0.1:3456", env = "DARX_DATA_PLAN_URL")]
    data_plan_url: String,

    #[clap(
        long,
        default_value = "127.0.0.1:3457",
        env = "DARX_CONTROL_PLAN_LISTEN_ADDR"
    )]
    control_plan_addr: SocketAddr,

    #[clap(
        long,
        default_value = "127.0.0.1:3457",
        env = "DARX_CONTROL_PLAN_URL"
    )]
    control_plan_url: String,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let registry = tracing_subscriber::registry();
    registry
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_filter(
                    tracing_subscriber::EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .init();

    let args = Cli::parse();
    let data =
        darx_data_plane::run_server(args.data_plan_addr, args.data_plane_dir)
            .await?;
    let control =
        darx_control_plane::run_server(args.control_plan_addr).await?;
    let (_, _) = futures::future::try_join(data, control).await?;
    Ok(())
}

pub fn setup_log() {}
