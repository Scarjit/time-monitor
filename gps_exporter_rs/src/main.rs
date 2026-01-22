#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
)]

mod gps;
mod metrics;
mod satellite;
mod server;

use metrics::GPSMetricsCollector;
use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::anyhow;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GPSD host
    #[arg(long, env = "GPSD_HOST", default_value = "127.0.0.1")]
    gpsd_host: String,

    /// GPSD port
    #[arg(long, env = "GPSD_PORT", default_value_t = 2947)]
    gpsd_port: u16,

    /// Port to expose metrics on
    #[arg(long, env = "EXPORTER_PORT", default_value_t = 9015)]
    exporter_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gps_exporter_rs=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    info!("Starting GPS Prometheus Exporter");
    info!("GPSD host: {}:{}", args.gpsd_host, args.gpsd_port);
    info!("Exporter port: {}", args.exporter_port);

    let socket_address: SocketAddr = match format!("{}:{}", args.gpsd_host, args.gpsd_port).to_socket_addrs()?.next() {
        None => {
            return Err(anyhow!("Cannot resolve exporter address"));
        }
        Some(v) => {v}
    };
    let collector = Arc::new(Mutex::new(GPSMetricsCollector::new(socket_address)?));

    // Spawn background task to update metrics
    let collector_clone = Arc::clone(&collector);
    tokio::spawn(async move {
        loop {
            if let Ok(mut c) = collector_clone.lock()
                && let Err(e) = c.update_metrics() {
                    warn!("Error updating metrics: {}", e);
                }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Build and start server
    let app = server::create_app(collector);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.exporter_port)).await?;
    info!("GPS exporter running on http://0.0.0.0:{}/metrics", args.exporter_port);

    axum::serve(listener, app).await?;

    Ok(())
}
