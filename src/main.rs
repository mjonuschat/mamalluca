use crate::moonraker::UpdateHandlerError;
use anyhow::Result;
use bytes::Bytes;
use clap::{ArgAction, ColorChoice, Parser};
use http_body_util::Full;
use hyper::body::Incoming as IncomingBody;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use moonraker::UpdateHandler;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinSet;
use tracing::{error, Level};

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

fn setup_exporter() -> Result<HttpExporterService> {
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder()?;

    Ok(HttpExporterService::new(handle))
}

#[derive(Clone)]
struct HttpExporterService {
    handle: PrometheusHandle,
}

impl HttpExporterService {
    pub fn new(handle: PrometheusHandle) -> Self {
        Self { handle }
    }
}

impl Service<Request<IncomingBody>> for HttpExporterService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        let handle = self.handle.clone();

        let res = match req.uri().path() {
            "/health" => mk_response("OK".into()),
            _ => mk_response(handle.render()),
        };

        Box::pin(async { res })
    }
}

async fn run(args: &Cli) -> Result<()> {
    let (handler, future) = UpdateHandler::new(&args.moonraker_url).await?;
    let handler = Arc::new(handler);

    let exporter = setup_exporter()?;
    let listener = TcpListener::bind(&args.prometheus_listen_address).await?;

    let mut set = JoinSet::new();

    // Start the HTTP server
    set.spawn({
        async move {
            loop {
                let (stream, _) = listener.accept().await?;
                let io = TokioIo::new(stream);
                let service = exporter.clone();

                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .keep_alive(false)
                        .serve_connection(io, service)
                        .await
                    {
                        error!("Failed to serve HTTP connection: {:?}", err)
                    }
                });
            }
        }
    });

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

    run(&args).await
}
