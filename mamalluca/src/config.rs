//! Application configuration.

use clap::{ArgAction, ColorChoice, Parser};
use std::net::SocketAddr;

/// Prometheus exporter for Klipper/Moonraker 3D printer metrics.
#[derive(Parser, Debug)]
#[clap(author, about, version, name = "mamalluca", color = ColorChoice::Auto)]
pub struct Cli {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    /// Moonraker WebSocket URL
    #[clap(short, long, default_value = "ws://127.0.0.1:7125/websocket")]
    pub moonraker_url: url::Url,
    /// HTTP listener address for Prometheus scraping
    #[clap(short, long, default_value = "0.0.0.0:9000")]
    pub prometheus_listen_address: SocketAddr,
}
