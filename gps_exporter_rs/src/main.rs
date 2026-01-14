mod gps;
mod metrics;
mod satellite;
mod server;

use metrics::GPSMetricsCollector;
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::anyhow;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Load configuration from environment with defaults
    let gpsd_host: String = env::var("GPSD_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()?;
    let gpsd_port: u16 = env::var("GPSD_PORT")
        .unwrap_or_else(|_| "2947".to_string())
        .parse()?;
    let exporter_port: u16 = env::var("EXPORTER_PORT")
        .unwrap_or_else(|_| "9015".to_string())
        .parse()?;

    info!("Starting GPS Prometheus Exporter");
    info!("GPSD host: {}:{}", gpsd_host, gpsd_port);
    info!("Exporter port: {}", exporter_port);

    let socket_address: SocketAddr = match format!("{}:{}", gpsd_host, gpsd_port).to_socket_addrs()?.next() {
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
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", exporter_port)).await?;
    info!("GPS exporter running on http://0.0.0.0:{}/metrics", exporter_port);

    axum::serve(listener, app).await?;

    Ok(())
}
