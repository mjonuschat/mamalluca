use crate::moonraker;
use crate::moonraker::types::Payload;
use crate::moonraker::{Client, MoonrakerCommands, MoonrakerStatusNotification};

use anyhow::anyhow;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex};
use url::Url;

#[derive(Error, Debug)]
pub enum UpdateHandlerError {
    #[error("Websocket update notification channel disconnected")]
    ChannelDisconnected,
    #[error("Update notification for `{0}` is not supported")]
    UnknownStatusUpdate(String),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Hash)]
enum StatusData {
    Mcu(String),
    Webhooks,
}

impl TryFrom<&str> for StatusData {
    type Error = UpdateHandlerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace().take(2);
        let kind = parts
            .next()
            .ok_or(UpdateHandlerError::UnknownStatusUpdate(value.to_owned()))?;

        match (kind, parts.next()) {
            ("mcu", None) => Ok(StatusData::Mcu("mcu".to_string())),
            ("mcu", Some(name)) => Ok(StatusData::Mcu(name.to_owned())),
            ("webhooks", _) => Ok(StatusData::Webhooks),
            _ => Err(UpdateHandlerError::UnknownStatusUpdate(value.to_owned())),
        }
    }
}

impl From<StatusData> for String {
    fn from(value: StatusData) -> Self {
        match value {
            StatusData::Mcu(name) => {
                if name == "mcu" {
                    String::from("mcu")
                } else {
                    format!("mcu {name}")
                }
            }
            StatusData::Webhooks => String::from("webhooks"),
        }
    }
}

pub struct UpdateHandler {
    initialized: AtomicBool,
    updates: Mutex<mpsc::Receiver<MoonrakerStatusNotification>>,
    connection: Arc<ezsockets::Client<Client>>,
    url: Url,
    current_status: DashMap<StatusData, serde_json::Value>,
}

impl UpdateHandler {
    pub async fn new(
        url: &Url,
        // objects: Option<Vec<String>>,
    ) -> anyhow::Result<(
        Self,
        impl std::future::Future<Output = std::result::Result<(), ezsockets::Error>>,
    )> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let (handle, future) = moonraker::Client::connect(url.as_str(), tx.clone()).await?;

        Ok((
            Self {
                initialized: AtomicBool::new(false),
                updates: Mutex::new(rx),
                connection: Arc::new(handle),
                url: url.to_owned(),
                current_status: DashMap::new(),
            },
            future,
        ))
    }

    pub async fn process(&self) -> Result<(), UpdateHandlerError> {
        let updates = &mut self.updates.lock().await;

        while let Some(ref notification) = updates.recv().await {
            let result = match notification {
                MoonrakerStatusNotification::MoonrakerConnected => {
                    self.on_moonraker_connected().await
                }
                MoonrakerStatusNotification::MoonrakerDisconnected => {
                    self.on_moonraker_disconnected().await
                }
                MoonrakerStatusNotification::KlipperStatusData(payload) => {
                    self.process_status_update(payload).await
                }
                n => {
                    tracing::info!(
                        "Implementation required for notification {}, {:#?}",
                        n,
                        &notification,
                    );
                    Ok(())
                }
            };
            if let Err(err) = result {
                tracing::error!(
                    "Processing status notification {} failed: {}",
                    &notification,
                    err
                );
            }
        }

        Err(UpdateHandlerError::ChannelDisconnected)
    }

    async fn process_status_update(&self, payload: &Payload) -> anyhow::Result<()> {
        if !payload.is_array() {
            anyhow::bail!("Malformed Klipper status update {:?}", payload);
        }

        if let Some(updates) = payload.as_array() {
            for update in updates {
                if let Some(update) = update.as_object() {
                    for (key, patch) in update {
                        let kind: StatusData = key.as_str().try_into()?;
                        // TODO: Separate into generic updatables and transformers...
                        tracing::debug!(key, "Processing status update");
                        let mut entry = self.current_status.entry(kind).or_insert(json!({}));
                        json_patch::merge(&mut entry, patch);
                    }
                }
            }
        }

        Ok(())
    }
    fn build_channel(
        &self,
    ) -> (
        oneshot::Sender<serde_json::Value>,
        oneshot::Receiver<serde_json::Value>,
    ) {
        oneshot::channel()
    }

    async fn on_moonraker_connected(&self) -> anyhow::Result<()> {
        tracing::info!(url = &self.url.to_string(), "Connected to Moonraker");
        let objects = self.get_object_list().await?;
        self.subscribe(objects).await?;

        Ok(())
    }

    async fn on_moonraker_disconnected(&self) -> anyhow::Result<()> {
        tracing::warn!(url = &self.url.to_string(), "Disconnected from Moonraker");
        self.initialized.store(false, Ordering::Relaxed);
        self.current_status.clear();

        Ok(())
    }

    async fn subscribe(&self, objects: Vec<StatusData>) -> anyhow::Result<()> {
        let (tx, rx) = self.build_channel();
        let objects = objects
            .into_iter()
            .map(|i| i.into())
            .collect::<Vec<String>>();
        self.connection
            .call(MoonrakerCommands::Subscribe((tx, objects)))?;

        let response = rx.await?;
        let updates = response
            .pointer("/result/status")
            .ok_or(anyhow!("Initial status updates not received"))?;

        self.current_status.clear();
        self.process_status_update(&json!([updates])).await?;

        self.initialized.store(true, Ordering::Relaxed);

        Ok(())
    }

    async fn get_object_list(&self) -> anyhow::Result<Vec<StatusData>> {
        let (tx, rx) = self.build_channel();
        self.connection.call(MoonrakerCommands::GetObjectList(tx))?;
        let response = rx.await?;

        Ok(response
            .pointer("/result/objects")
            .and_then(|v| v.as_array())
            .map(|v| {
                v.iter()
                    .filter_map(|o| o.as_str())
                    .map(|v| v.try_into())
                    .filter_map(Result::ok)
                    .collect::<Vec<StatusData>>()
            })
            .unwrap_or_default())
    }
}
