use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser};
use moonraker::UpdateHandler;
use tracing::Level;

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    setup_logging(args.verbose)?;

    let (mut handler, future) = UpdateHandler::new(&args.moonraker_url).await?;
    tokio::spawn(async move { handler.process().await });

    // todo!();
    future.await.unwrap();
    Ok(())
}
