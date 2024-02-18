use crate::types::MetricsExporter;
use metrics::{counter, describe_counter, gauge, Unit};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum KlippyState {
    Ready,
    Error,
    Shutdown,
    Startup,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct McuStats {
    #[serde(default)]
    adj: u64,
    #[serde(default)]
    bytes_invalid: u64,
    #[serde(default)]
    bytes_read: u64,
    #[serde(default)]
    bytes_retransmit: u64,
    bytes_write: u64,
    freq: u64,
    mcu_awake: f64,
    mcu_task_avg: f64,
    mcu_task_stddev: f64,
    ready_bytes: u64,
    upcoming_bytes: u64,
    send_seq: u64,
    receive_seq: u64,
    retransmit_seq: u64,
    srtt: f64,
    rto: f64,
    rttvar: f64,
}

impl MetricsExporter for McuStats {
    fn describe(&self) {
        describe_counter!("klipper.stats.mcu.bytes_invalid", Unit::Bytes, "");
        describe_counter!("klipper.stats.mcu.bytes_read", Unit::Bytes, "");
        describe_counter!("klipper.stats.mcu.bytes_write", Unit::Bytes, "");
        describe_counter!("klipper.stats.mcu.bytes_retransmit", Unit::Bytes, "");
        describe_counter!("klipper.stats.mcu.ready_bytes", Unit::Bytes, "");
        describe_counter!("klipper.stats.mcu.upcomping_bytes", Unit::Bytes, "");
    }

    fn export(&self, name: Option<&String>) {
        let mut labels = Vec::new();
        if let Some(name) = name {
            labels.push(("name", name.to_owned()));
        }

        gauge!("klipper.stats.mcu.adj", &labels).set(self.adj as f64);
        gauge!("klipper.stats.mcu.freq", &labels).set(self.freq as f64);
        gauge!("klipper.stats.mcu.mcu_awake", &labels).set(self.mcu_awake);
        gauge!("klipper.stats.mcu.mcu_task_avg", &labels).set(self.mcu_task_avg);
        gauge!("klipper.stats.mcu.mcu_task_stddev", &labels).set(self.mcu_task_stddev);
        gauge!("klipper.stats.mcu.ready_bytes", &labels).set(self.ready_bytes as f64);
        gauge!("klipper.stats.mcu.upcoming_bytes", &labels).set(self.upcoming_bytes as f64);

        counter!("klipper.stats.mcu.bytes_read", &labels).absolute(self.bytes_read);
        counter!("klipper.stats.mcu.bytes_write", &labels).absolute(self.bytes_write);
        counter!("klipper.stats.mcu.bytes_invalid", &labels).absolute(self.bytes_invalid);
        counter!("klipper.stats.mcu.bytes_retransmit", &labels).absolute(self.bytes_retransmit);

        counter!("klipper.stats.mcu.receive_seq", &labels).absolute(self.receive_seq);
        counter!("klipper.stats.mcu.send_seq", &labels).absolute(self.send_seq);
        counter!("klipper.stats.mcu.retransmit_seq", &labels).absolute(self.retransmit_seq);

        gauge!("klipper.stats.mcu.rto", &labels).set(self.rto);
        gauge!("klipper.stats.mcu.rttvar", &labels).set(self.rttvar);
        gauge!("klipper.stats.mcu.srtt", &labels).set(self.srtt);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WebhooksStats {
    /// The current printer state
    state: KlippyState,
    /// The current state message
    state_message: String,
}

impl MetricsExporter for WebhooksStats {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct HeaterInformation {
    available_heaters: HashSet<String>,
    available_sensors: HashSet<String>,
    available_monitors: HashSet<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ExtruderStats {
    can_extrude: bool,
    power: f64,
    pressure_advance: f64,
    smooth_time: f64,
    target: f64,
    temperature: f64,
    time_offset: f64,
}

impl MetricsExporter for ExtruderStats {
    fn describe(&self) {
        describe_counter!("klipper.stats.extruder.smooth_time", Unit::Seconds, "");
    }

    fn export(&self, name: Option<&String>) {
        let mut labels = Vec::new();
        if let Some(name) = name {
            labels.push(("name", name.to_owned()));
        }

        gauge!("klipper.stats.extruder.can_extrude", &labels).set(self.can_extrude as u8 as f64);
        gauge!("klipper.stats.extruder.power", &labels).set(self.power);
        gauge!("klipper.stats.extruder.pressure_advance", &labels).set(self.pressure_advance);
        gauge!("klipper.stats.extruder.smooth_tmime", &labels).set(self.smooth_time);
        gauge!("klipper.stats.extruder.target", &labels).set(self.target);
        gauge!("klipper.stats.extruder.temperature", &labels).set(self.temperature);
        gauge!("klipper.stats.extruder.time_offset", &labels).set(self.time_offset);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct HeaterBedStats {
    power: f64,
    target: f64,
    temperature: f64,
}

impl MetricsExporter for HeaterBedStats {
    fn export(&self, name: Option<&String>) {
        let mut labels = Vec::new();
        if let Some(name) = name {
            labels.push(("name", name.to_owned()));
        }

        gauge!("klipper.stats.heater_bed.power", &labels).set(self.power);
        gauge!("klipper.stats.heater_bed.target", &labels).set(self.target);
        gauge!("klipper.stats.heater_bed.temperature", &labels).set(self.temperature);
    }
}
