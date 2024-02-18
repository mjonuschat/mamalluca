use crate::moonraker::types::JsonRPCRequest;
use crate::moonraker::Payload;

use async_trait::async_trait;
use dashmap::DashMap;
use ezsockets::client::ClientCloseMode;
use ezsockets::{ClientConfig, CloseFrame, Error};
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tokio::sync::oneshot::Sender;
use url::Url;
type ConnectionID = u64;
#[derive(Debug)]
pub(crate) enum MoonrakerCommands {
    GetObjectList(Sender<serde_json::Value>),
    Subscribe((Sender<serde_json::Value>, Vec<String>)),
}

#[derive(Debug, strum::Display)]
pub(crate) enum MoonrakerStatusNotification {
    MoonrakerConnected,
    MoonrakerDisconnected,
    KlippyReady,
    KlippyShutdown,
    KlippyDisconnected,
    KlipperStatusData(Payload),
    MoonrakerStatusData(Payload),
}

#[derive(Debug)]
struct MoonrakerClientState {
    requests: DashMap<ConnectionID, Sender<serde_json::Value>>,
    next_id: AtomicU64,
}

#[derive(Debug)]
pub(crate) struct Client {
    handle: ezsockets::Client<Self>,
    updates: mpsc::Sender<MoonrakerStatusNotification>,
    state: MoonrakerClientState,
}

impl Client {
    fn new(
        connection: ezsockets::Client<Self>,
        updates: mpsc::Sender<MoonrakerStatusNotification>,
    ) -> Self {
        Self {
            handle: connection,
            updates,
            state: MoonrakerClientState {
                requests: DashMap::new(),
                next_id: AtomicU64::new(0),
            },
        }
    }

    pub async fn connect(
        url: &str,
        updates: mpsc::Sender<MoonrakerStatusNotification>,
    ) -> anyhow::Result<(
        ezsockets::Client<Client>,
        impl Future<Output = Result<(), ezsockets::Error>>,
    )> {
        let url = Url::parse(url)?;
        let config = ClientConfig::new(url);
        Ok(ezsockets::connect(|handle| Client::new(handle, updates), config).await)
    }

    async fn process_call_response(&self, response: serde_json::Value) {
        let conn_id = response.get("id").and_then(|v| v.as_u64());
        if let Some(conn_id) = conn_id {
            if let Some((_, tx)) = self.state.requests.remove(&conn_id) {
                if let Err(msg) = tx.send(response) {
                    eprintln!("Error returning response for {}: {:?}", conn_id, msg)
                }
            }
        }
    }

    async fn process_notification(&self, response: serde_json::Value) {
        if let Some(method) = response.get("method") {
            let payload = response.get("params").unwrap_or(&json!({})).to_owned();

            // TODO: thiserror
            let notification = match method.as_str() {
                Some("notify_proc_stat_update") => {
                    Some(MoonrakerStatusNotification::MoonrakerStatusData(payload))
                }
                Some("notify_status_update") => {
                    Some(MoonrakerStatusNotification::KlipperStatusData(payload))
                }
                Some("notify_klippy_ready") => Some(MoonrakerStatusNotification::KlippyReady),
                Some("notify_klippy_shutdown") => Some(MoonrakerStatusNotification::KlippyShutdown),
                Some("notify_klippy_disconnected") => {
                    Some(MoonrakerStatusNotification::KlippyDisconnected)
                }
                Some(method) => {
                    // notify_sensor_update
                    // notify_service_state_changed
                    // notify_update_refreshed
                    eprintln!("Unknown status notification: {}", method);
                    None
                }
                None => {
                    eprintln!("Null method name in response: {:#?}", response);
                    None
                }
            };

            if let Some(notification) = notification {
                if let Err(err) = self.updates.send(notification).await {
                    eprintln!("Error sending notification to update handler: {}", err)
                }
            }
        }
    }
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = MoonrakerCommands;

    async fn on_text(&mut self, text: String) -> anyhow::Result<(), ezsockets::Error> {
        let response = serde_json::from_str(&text).unwrap_or(json!({}));

        if response.get("method").is_none() {
            self.process_call_response(response).await
        } else {
            self.process_notification(response).await
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> anyhow::Result<(), ezsockets::Error> {
        tracing::info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> anyhow::Result<(), ezsockets::Error> {
        match call {
            MoonrakerCommands::GetObjectList(tx) => {
                let next_id = self.state.next_id.fetch_add(1, Ordering::Relaxed);

                let request = JsonRPCRequest::new("printer.objects.list", next_id);
                self.state.requests.insert(next_id, tx);
                self.handle.text(serde_json::to_string(&request)?)?;
            }
            MoonrakerCommands::Subscribe((tx, objects)) => {
                let next_id = self.state.next_id.fetch_add(1, Ordering::Relaxed);
                let wanted = objects
                    .iter()
                    .map(|v| (v, None))
                    .collect::<HashMap<_, Option<Vec<String>>>>();
                let mut request = JsonRPCRequest::new("printer.objects.subscribe", next_id);
                request.params = json!({
                    "objects": wanted,
                });
                self.state.requests.insert(next_id, tx);
                self.handle.text(serde_json::to_string(&request)?)?;
            }
        }
        Ok(())
    }

    /// Called when the client successfully connected (or reconnected).
    ///
    /// Returning an error will force-close the client.
    async fn on_connect(&mut self) -> Result<(), Error> {
        if let Err(err) = self
            .updates
            .send(MoonrakerStatusNotification::MoonrakerConnected)
            .await
        {
            eprintln!("Error sending connect notification {:#?}", err);
        }
        Ok(())
    }

    /// Called when the connection is closed by the server.
    ///
    /// Returning an error will force-close the client.
    ///
    /// By default, the client will try to reconnect. Return [`ClientCloseMode::Close`] here to fully close instead.
    ///
    /// For reconnections, use `ClientConfig::reconnect_interval`.
    async fn on_close(&mut self, _frame: Option<CloseFrame>) -> Result<ClientCloseMode, Error> {
        if let Err(err) = self
            .updates
            .send(MoonrakerStatusNotification::MoonrakerDisconnected)
            .await
        {
            eprintln!("Error sending connect notification {:#?}", err);
        }
        Ok(ClientCloseMode::Reconnect)
    }

    /// Called when the connection is closed by the socket dying.
    ///
    /// Returning an error will force-close the client.
    ///
    /// By default, the client will try to reconnect. Return [`ClientCloseMode::Close`] here to fully close instead.
    ///
    /// For reconnections, use `ClientConfig::reconnect_interval`.
    async fn on_disconnect(&mut self) -> Result<ClientCloseMode, Error> {
        if let Err(err) = self
            .updates
            .send(MoonrakerStatusNotification::MoonrakerDisconnected)
            .await
        {
            eprintln!("Error sending disconnect notification {:#?}", err);
        }

        Ok(ClientCloseMode::Reconnect)
    }
}
