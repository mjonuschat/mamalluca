use crate::types::MetricsExporter;
use metrics::{counter, describe_counter, gauge, Unit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct MoonrakerStats {
    cpu_temp: f64,
    moonraker_stats: MoonrakerServiceData,
    network: HashMap<String, NetworkInterfaceData>,
    system_cpu_usage: SystemCpuUsageData,
    system_memory: SystemMemoryUsageData,
    websocket_connections: u64,
}

impl MetricsExporter for MoonrakerStats {
    fn describe(&self) {
        describe_counter!("moonraker.stats.service.memory", Unit::Kibibytes, "");
        describe_counter!(
            "moonraker.stats.network.bandwidth",
            Unit::KilobitsPerSecond,
            ""
        );
        describe_counter!("moonraker.stats.network.rx_bytes", Unit::Bytes, "");
        describe_counter!("moonraker.stats.network.tx_bytes", Unit::Bytes, "");
        describe_counter!("moonraker.stats.system.cpu_usage", Unit::Percent, "");
        describe_counter!("moonraker.stats.system.memory_available", Unit::Bytes, "");
        describe_counter!("moonraker.stats.system.memory_total", Unit::Bytes, "");
        describe_counter!("moonraker.stats.system.memory_used", Unit::Bytes, "");
    }

    fn export(&self, name: Option<&String>) {
        let mut labels = Vec::new();
        if let Some(name) = name {
            labels.push(("name", name.to_owned()));
        }

        // Moonraker Service
        gauge!("moonraker.stats.service.memory", &labels).set(self.moonraker_stats.memory as f64);
        gauge!("moonraker.stats.service.cpu_usage", &labels).set(self.moonraker_stats.cpu_usage);
        gauge!("moonraker.stats.service.websocket_connections", &labels)
            .set(self.websocket_connections as f64);

        // Network interface metrics
        for (intf, data) in &self.network {
            let intf_labels: Vec<_> = labels
                .clone()
                .into_iter()
                .chain([("interface", intf.to_owned())])
                .collect();

            gauge!("moonraker.stats.network.bandwidth", &intf_labels).set(data.bandwidth);

            counter!("moonraker.stats.network.rx_bytes", &intf_labels).absolute(data.rx_bytes);
            counter!("moonraker.stats.network.rx_drop", &intf_labels).absolute(data.rx_drop);
            counter!("moonraker.stats.network.rx_errs", &intf_labels).absolute(data.rx_errs);
            counter!("moonraker.stats.network.rx_packets", &intf_labels).absolute(data.rx_packets);

            counter!("moonraker.stats.network.tx_bytes", &intf_labels).absolute(data.tx_bytes);
            counter!("moonraker.stats.network.tx_drop", &intf_labels).absolute(data.tx_drop);
            counter!("moonraker.stats.network.tx_errs", &intf_labels).absolute(data.tx_errs);
            counter!("moonraker.stats.network.tx_packets", &intf_labels).absolute(data.tx_packets);
        }

        // Average CPU usage metric
        {
            let cpu_labels: Vec<_> = labels
                .clone()
                .into_iter()
                .chain([("cpu", String::from("cpu"))])
                .collect();
            gauge!("moonraker.stats.system.cpu_usage", &cpu_labels).set(self.system_cpu_usage.cpu);
            gauge!("moonraker.stats.system.cpu_temp", &cpu_labels).set(self.cpu_temp);
        }

        // Per CPU core usage metrics
        for (core, value) in &self.system_cpu_usage.cores {
            let core_labels: Vec<_> = labels
                .clone()
                .into_iter()
                .chain([("cpu", core.to_owned())])
                .collect();
            gauge!("moonraker.stats.system.cpu_usage", &core_labels).set(*value);
        }

        // Memory usage metrics
        gauge!("moonraker.stats.system.memory_total", &labels).set(self.system_memory.total as f64);
        gauge!("moonraker.stats.system.memory_available", &labels)
            .set(self.system_memory.available as f64);
        gauge!("moonraker.stats.system.memory_used", &labels).set(self.system_memory.used as f64);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct MoonrakerServiceData {
    cpu_usage: f64,
    mem_units: String,
    memory: u64,
    time: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct NetworkInterfaceData {
    bandwidth: f64,
    rx_bytes: u64,
    rx_drop: u64,
    rx_errs: u64,
    rx_packets: u64,
    tx_bytes: u64,
    tx_drop: u64,
    tx_errs: u64,
    tx_packets: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SystemCpuUsageData {
    cpu: f64,
    #[serde(flatten)]
    cores: HashMap<String, f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SystemMemoryUsageData {
    available: u64,
    total: u64,
    used: u64,
}
