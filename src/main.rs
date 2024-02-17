use crate::moonraker::UpdateHandlerError;
use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser};
use metrics_exporter_prometheus::PrometheusBuilder;
use moonraker::UpdateHandler;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::Level;
use url::Url;

mod moonraker;
mod types;

/// Prometheus exporter for Moonraker.
#[derive(clap::Parser, Debug)]
#[clap(author, about, version, name = "mamalluca", color=ColorChoice::Auto)]
pub(crate) struct Cli {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action=ArgAction::Count)]
    verbose: u8,
    /// Moonraker URL
    #[clap(short, long, default_value = "ws://127.0.0.1:7125/websocket")]
    moonraker_url: url::Url,
    /// Prometheus Listener Socket
    #[clap(short, long, default_value = "0.0.0.0:9000")]
    prometheus_listen_address: SocketAddr,
}

fn setup_logging(verbose: u8) -> Result<()> {
    let log_level = match verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    // Logging
    tracing_subscriber::fmt().with_max_level(log_level).init();

    Ok(())
}

fn setup_exporter(addr: &SocketAddr) -> Result<()> {
    let builder = PrometheusBuilder::new().with_http_listener(addr.to_owned());
    builder.install()?;

    Ok(())
}

async fn run(moonraker_url: &Url) -> Result<()> {
    let (handler, future) = UpdateHandler::new(moonraker_url).await?;
    let handler = Arc::new(handler);

    let mut set = JoinSet::new();

    // Start the update handler
    set.spawn({
        let handler = handler.clone();
        async move { handler.process().await }
    });

    // Start the periodic metrics update
    set.spawn({
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        async move {
            loop {
                interval.tick().await;
                handler.export().await?;
            }
        }
    });

    set.spawn(async move {
        future
            .await
            .map_err(|_e| UpdateHandlerError::FatalMoonrakerConnectionError)
    });

    // Wait for the first task to exit
    if let Some(result) = set.join_next().await {
        result??
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    setup_logging(args.verbose)?;
    setup_exporter(&args.prometheus_listen_address)?;

    run(&args.moonraker_url).await
}
