//! Mamalluca -- Prometheus exporter for Klipper/Moonraker metrics.
//!
//! Connects to a Moonraker instance via WebSocket, subscribes to printer
//! status updates, and serves them as Prometheus metrics over HTTP.

mod config;
mod metrics;
mod server;

use anyhow::Result;
use clap::Parser;
use config::Cli;
use metrics::registry::CollectorRegistry;
use moonraker_client::{MoonrakerClient, MoonrakerConfig, MoonrakerEvent};
use server::AppState;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_util::sync::CancellationToken;
use tracing::Level;

/// Set up the tracing subscriber with the verbosity level from CLI flags.
///
/// Maps `-v` count to log level: 0 = WARN, 1 = INFO, 2 = DEBUG, 3+ = TRACE.
fn setup_logging(verbose: u8) -> Result<()> {
    let log_level = match verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    setup_logging(args.verbose)?;

    let cancel = CancellationToken::new();

    // Signal handler for graceful shutdown (SIGTERM + SIGINT).
    // Spawned as a background task so the main logic can proceed.
    let shutdown_token = cancel.clone();
    tokio::spawn(async move {
        let ctrl_c = tokio::signal::ctrl_c();

        // On Unix, also handle SIGTERM for systemd / container compatibility.
        #[cfg(unix)]
        {
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to register SIGTERM handler");
            tokio::select! {
                _ = ctrl_c => {},
                _ = sigterm.recv() => {},
            }
        }
        #[cfg(not(unix))]
        {
            ctrl_c.await.ok();
        }

        tracing::info!("Received shutdown signal");
        shutdown_token.cancel();
    });

    // Install the Prometheus metrics recorder globally.
    // The returned handle lets us render the text exposition format on demand.
    let metrics_handle =
        metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder()?;

    // Build the collector registry from all `#[collector]`-annotated types.
    let registry = Arc::new(CollectorRegistry::from_inventory());

    // Shared flag so the /health endpoint can report connection status.
    let connection_status = Arc::new(AtomicBool::new(false));

    // Connect to Moonraker (spawns a background reconnect loop).
    let config = MoonrakerConfig {
        url: args.moonraker_url.clone(),
        ..MoonrakerConfig::default()
    };
    let (client, mut events) = MoonrakerClient::connect(config, cancel.clone()).await?;

    // Build the axum HTTP server.
    let state = AppState {
        metrics_handle,
        connection_status: connection_status.clone(),
    };
    let app = server::app(state);
    let listener = tokio::net::TcpListener::bind(&args.prometheus_listen_address).await?;
    tracing::info!(
        address = %args.prometheus_listen_address,
        "HTTP server listening"
    );

    // Run HTTP server and event processor concurrently until shutdown.
    // Clone cancel before `cancelled_owned()` consumes it.
    let event_cancel = cancel.clone();
    tokio::select! {
        result = axum::serve(listener, app)
            .with_graceful_shutdown(cancel.cancelled_owned()) => {
            if let Err(e) = result {
                tracing::error!(error = %e, "HTTP server error");
            }
        }
        _ = process_events(
            &mut events,
            &client,
            &registry,
            &connection_status,
            event_cancel,
        ) => {}
    }

    client.close().await;
    Ok(())
}

/// Process Moonraker events and dispatch status updates to collectors.
///
/// Runs until the cancellation token fires or the event channel closes.
/// On connect, queries available printer objects and subscribes to all of them.
async fn process_events(
    events: &mut tokio::sync::mpsc::Receiver<MoonrakerEvent>,
    client: &MoonrakerClient,
    registry: &CollectorRegistry,
    connection_status: &AtomicBool,
    cancel: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            event = events.recv() => {
                let Some(event) = event else { break };
                match event {
                    MoonrakerEvent::Connected => {
                        tracing::info!("Connected to Moonraker");
                        connection_status.store(true, Ordering::Relaxed);

                        // Discover available printer objects and subscribe.
                        match client.get_object_list().await {
                            Ok(objects) => {
                                if let Err(e) =
                                    client.subscribe(&objects).await
                                {
                                    tracing::error!(
                                        error = %e,
                                        "Failed to subscribe"
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    "Failed to get object list"
                                );
                            }
                        }
                    }
                    MoonrakerEvent::Disconnected { reason } => {
                        tracing::warn!(
                            ?reason,
                            "Disconnected from Moonraker"
                        );
                        connection_status.store(false, Ordering::Relaxed);
                    }
                    MoonrakerEvent::StatusUpdate { key, data } => {
                        if let Err(e) = registry.dispatch(&key, &data) {
                            tracing::debug!(
                                key,
                                error = %e,
                                "Failed to process status update"
                            );
                        }
                    }
                    MoonrakerEvent::KlippyStateChanged(state) => {
                        tracing::info!(
                            ?state,
                            "Klippy state changed"
                        );
                    }
                }
            }
        }
    }
}
