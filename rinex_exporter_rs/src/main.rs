#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
)]

mod decompress;
mod download;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to push metrics to
    #[arg(long, env = "PUSH_GATEWAY_PORT", default_value_t = 9015)]
    push_gateway_port: u16,

    /// Host to push metrics to
    #[arg(long, env = "PUSH_GATEWAY_HOST", default_value = "127.0.0.1")]
    push_gateway_host: String,

    /// GNSS Station to observe
    #[arg(long, env = "STATION", default_value = "EUSK00DEU")]
    station: String,

    /// GNSS Station observation duration
    #[arg(long, env = "OBSERVATION_DURATION", default_value = "01D")]
    observation_duration: String,

    /// GNSS Station observation sample interval
    #[arg(long, env = "OBSERVATION_INTERVAL", default_value = "30S")]
    observation_interval: String,

    /// Backfill days
    #[arg(long, env = "BACKFILL_DAYS", default_value = "5")]
    backfill_days: u32,

    /// Data directory
    #[arg(long, env = "DATA_DIR", default_value = "./data")]
    data_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rinex_exporter_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    info!("Starting RINEX Prometheus Exporter");
    info!(
        "Push Gateway host: {}:{}",
        args.push_gateway_host, args.push_gateway_port
    );
    info!("Station: {}", args.station);
    info!("Backfill days: {}", args.backfill_days);

    download::download_rinex_data(
        &args.station,
        args.backfill_days,
        &args.observation_duration,
        &args.observation_interval,
        &args.data_dir,
    )
    .await?;

    decompress::uncompress_rinex_data(&args.data_dir).await?;

    Ok(())
}
