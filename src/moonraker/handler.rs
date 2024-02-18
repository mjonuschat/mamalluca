use crate::moonraker::types::Payload;
use crate::moonraker::{Client, MoonrakerCommands, MoonrakerStatusNotification};

use crate::types::{klipper, moonraker, MetricsExporter};
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
pub(crate) enum UpdateHandlerError {
    #[error("Websocket update notification channel disconnected")]
    ChannelDisconnected,
    #[error("Update notification for `{0}` is not supported")]
    UnknownStatusUpdate(String),
    #[error("Error deserializing stats data")]
    DeserializationError(#[from] serde_json::Error),
    #[error("Required field not found: `{0}`")]
    MissingStatsField(String),
    #[error("Fatal Moonraker connection error")]
    FatalMoonrakerConnectionError,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Hash)]
enum StatusData {
    ControllerFan(String),
    ExcludeObject,
    Extruder(String),
    Fan(String),
    FanGeneric(String),
    FilamentMotionSensor(String),
    FilamentSwitchSensor(String),
    GCodeMove,
    HeaterBed(String),
    HeaterFan(String),
    Mcu(String),
    MoonrakerStatus,
    MotionReport,
    PauseResume,
    PrintStats,
    Probe,
    StepperEnable,
    TemperatureSensor(String),
    TMC2130(String),
    TMC2208(String),
    TMC2209(String),
    TMC2240(String),
    TMC2660(String),
    TMC5160(String),
    Toolhead,
    VirtualSdCard,
    Webhooks,
    ZThermalAdjust,
    ZTilt,
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
            ("extruder", Some(name)) => Ok(StatusData::Extruder(name.to_owned())),
            ("extruder", None) => Ok(StatusData::Extruder("extruder".to_owned())),
            ("heater_bed", Some(name)) => Ok(StatusData::HeaterBed(name.to_owned())),
            ("heater_bed", None) => Ok(StatusData::HeaterBed("heater_bed".to_owned())),
            ("temperature_sensor", Some(name)) => {
                Ok(StatusData::TemperatureSensor(name.to_owned()))
            }
            ("controller_fan", Some(name)) => Ok(StatusData::ControllerFan(name.to_owned())),
            ("tmc2130", Some(name)) => Ok(StatusData::TMC2130(name.to_owned())),
            ("tmc2208", Some(name)) => Ok(StatusData::TMC2208(name.to_owned())),
            ("tmc2209", Some(name)) => Ok(StatusData::TMC2209(name.to_owned())),
            ("tmc2240", Some(name)) => Ok(StatusData::TMC2240(name.to_owned())),
            ("tmc2660", Some(name)) => Ok(StatusData::TMC2660(name.to_owned())),
            ("tmc5160", Some(name)) => Ok(StatusData::TMC5160(name.to_owned())),
            ("stepper_enable", _) => Ok(StatusData::StepperEnable),
            ("fan", _) => Ok(StatusData::Fan(String::from("fan"))),
            ("fan_generic", Some(name)) => Ok(StatusData::FanGeneric(name.to_owned())),
            ("heater_fan", Some(name)) => Ok(StatusData::HeaterFan(name.to_owned())),
            ("z_thermal_adjust", _) => Ok(StatusData::ZThermalAdjust),
            ("filament_motion_sensor", Some(name)) => {
                Ok(StatusData::FilamentMotionSensor(name.to_owned()))
            }
            ("filament_switch_sensor", Some(name)) => {
                Ok(StatusData::FilamentSwitchSensor(name.to_owned()))
            }
            ("pause_resume", _) => Ok(StatusData::PauseResume),
            ("probe", _) => Ok(StatusData::Probe),
            ("z_tilt", _) => Ok(StatusData::ZTilt),
            ("motion_report", _) => Ok(StatusData::MotionReport),
            ("exclude_object", _) => Ok(StatusData::ExcludeObject),
            ("toolhead", _) => Ok(StatusData::Toolhead),
            ("gcode_move", _) => Ok(StatusData::GCodeMove),
            ("print_stats", _) => Ok(StatusData::PrintStats),
            ("virtual_sdcard", _) => Ok(StatusData::VirtualSdCard),
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
            StatusData::MoonrakerStatus => String::from("moonraker"),
            StatusData::Extruder(name) => {
                if name == "extruder" {
                    String::from("extruder")
                } else {
                    format!("extruderÏ {name}")
                }
            }
            StatusData::HeaterBed(name) => {
                if name == "heater_bed" {
                    String::from("heater_bed")
                } else {
                    format!("heater_bed {name}")
                }
            }
            StatusData::TemperatureSensor(name) => {
                format!("temperature_sensor {name}")
            }
            StatusData::ControllerFan(name) => {
                format!("controller_fan {name}")
            }
            StatusData::TMC2130(name) => {
                format!("tmc2130 {name}")
            }
            StatusData::TMC2208(name) => {
                format!("tmc2208 {name}")
            }
            StatusData::TMC2209(name) => {
                format!("tmc2209 {name}")
            }
            StatusData::TMC2240(name) => {
                format!("tmc2240 {name}")
            }
            StatusData::TMC2660(name) => {
                format!("tmc2660 {name}")
            }
            StatusData::TMC5160(name) => {
                format!("tmc5160 {name}")
            }
            StatusData::StepperEnable => String::from("stepper_enable"),
            StatusData::Fan(_) => String::from("fan"),
            StatusData::FanGeneric(name) => {
                format!("fan_generic {name}")
            }
            StatusData::HeaterFan(name) => {
                format!("heater_fan {name}")
            }
            StatusData::ZThermalAdjust => String::from("z_thermal_adjust"),
            StatusData::FilamentSwitchSensor(name) => {
                format!("filament_switch_sensor {name}")
            }
            StatusData::FilamentMotionSensor(name) => {
                format!("filament_motion_sensor {name}")
            }
            StatusData::PauseResume => String::from("pause_resume"),
            StatusData::Probe => String::from("probe"),
            StatusData::ZTilt => String::from("z_tilt"),
            StatusData::MotionReport => String::from("motion_report"),
            StatusData::ExcludeObject => String::from("exclude_object"),
            StatusData::Toolhead => String::from("toolhead"),
            StatusData::GCodeMove => String::from("gcode_move"),
            StatusData::PrintStats => String::from("print_stats"),
            StatusData::VirtualSdCard => String::from("virtual_sdcard"),
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
        let (handle, future) = Client::connect(url.as_str(), tx.clone()).await?;

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

    pub async fn export(&self) -> Result<(), UpdateHandlerError> {
        let current_status = self.current_status.clone().into_read_only();
        for (data_type, data) in current_status.iter() {
            let mut name = None;
            let exporter: Box<dyn MetricsExporter> = match data_type {
                StatusData::Mcu(identifier) => {
                    name.replace(identifier);
                    let data = data.pointer("/last_stats").ok_or(
                        UpdateHandlerError::MissingStatsField(format!(
                            "mcu.{identifier}.last_stats"
                        )),
                    )?;
                    let data: klipper::McuStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::Webhooks => {
                    let data: klipper::WebhooksStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::MoonrakerStatus => {
                    tracing::debug!(key = "moonraker", "Processing status update");
                    let data = data
                        .pointer("/0")
                        .ok_or(UpdateHandlerError::MissingStatsField(
                            "moonraker.status".to_string(),
                        ))?;
                    let data: moonraker::MoonrakerStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::Extruder(identifier) => {
                    name.replace(identifier);
                    let data: klipper::ExtruderStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::HeaterBed(identifier) => {
                    name.replace(identifier);
                    let data: klipper::HeaterBedStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::TemperatureSensor(identifier) => {
                    name.replace(identifier);
                    let data: klipper::TemperatureSensorStats =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::ControllerFan(identifier) => {
                    name.replace(identifier);
                    let data: klipper::GenericFanStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::TMC2130(identifier)
                | StatusData::TMC2208(identifier)
                | StatusData::TMC2209(identifier)
                | StatusData::TMC2240(identifier)
                | StatusData::TMC2660(identifier)
                | StatusData::TMC5160(identifier) => {
                    name.replace(identifier);
                    let data: klipper::TMCStepperMotorDriver =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::StepperEnable => {
                    let data: klipper::StepperEnableStats =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::Fan(identifier)
                | StatusData::FanGeneric(identifier)
                | StatusData::HeaterFan(identifier) => {
                    name.replace(identifier);
                    let data: klipper::GenericFanStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::ZThermalAdjust => {
                    let data: klipper::ZThermalAdjustStats =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::FilamentMotionSensor(identifier)
                | StatusData::FilamentSwitchSensor(identifier) => {
                    name.replace(identifier);

                    let data: klipper::FilamentRunoutSensorStats =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::PauseResume => {
                    let data: klipper::PauseResumeStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::Probe => {
                    let data: klipper::ProbeStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::ZTilt => {
                    let data: klipper::ZTiltStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::MotionReport => {
                    let data: klipper::MotionReportStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::ExcludeObject => {
                    let data: klipper::ExcludeObjectStats =
                        serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::Toolhead => {
                    let data: klipper::ToolheadStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::GCodeMove => {
                    let data: klipper::GCodeMoveStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::PrintStats => {
                    let data: klipper::PrintStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
                StatusData::VirtualSdCard => {
                    let data: klipper::VirtualSdCardStats = serde_json::from_value(data.to_owned())?;
                    Box::new(data)
                }
            };
            exporter.export(name)
        }
        Ok(())
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
                MoonrakerStatusNotification::MoonrakerStatusData(payload) => {
                    self.current_status
                        .insert(StatusData::MoonrakerStatus, payload.to_owned());
                    Ok(())
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
