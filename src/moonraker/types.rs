use serde::{Deserialize, Serialize};
use serde_json::json;
use std::string::ToString;

pub(crate) type Payload = serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct JsonRPCRequest {
    method: String,
    #[serde(default)]
    id: u64,
    jsonrpc: String,
    pub(crate) params: Payload,
}

impl JsonRPCRequest {
    pub fn new(method: &str, id: u64) -> Self {
        Self {
            method: method.to_string(),
            id,
            jsonrpc: "2.0".to_string(),
            params: json!({}),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct JsonRPCError {
    code: usize,
    message: String,
    data: Payload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct JsonRPCResponse {
    id: usize,
    result: Option<Payload>,
    error: Option<JsonRPCError>,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub(crate) struct ObjectList {
//     objects: Vec<String>,
// }
//
// impl ObjectList {
//     const SUPPORTED: &'static [&'static str] = &["toolhead"];
//
//     pub fn new(value: JsonRPCResponse) -> anyhow::Result<Self> {
//         if let Some(raw_objects) = value
//             .result
//             .and_then(|v| v.get("objects").and_then(|v| v.as_array()).cloned())
//         {
//             return Ok(Self {
//                 objects: raw_objects
//                     .into_iter()
//                     .filter_map(|v| v.as_str().map(|v| v.to_owned()))
//                     .collect(),
//             });
//         }
//
//         anyhow::bail!("Error parsing response into object list")
//     }
//
//     pub fn wanted(&self) -> Vec<String> {
//         self.objects
//             .iter()
//             .filter(|item| Self::SUPPORTED.contains(&&***item))
//             .map(String::from)
//             .collect()
//     }
// }

// /// Moonraker Stats Data
// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct MoonrakerStatsData {
//     time: f64,
//     cpu_usage: f64,
//     memory: i64,
//     mem_units: String,
// }
//
// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct MoonrakerNetworkInterfaceStatsData {
//     rx_bytes: i64,
//     tx_bytes: i64,
//     bandwidth: f64,
// }
// /// Moonraker Process Statistics Update Data
// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct MoonrakerProcessStatisticData {
//     moonraker_stats: MoonrakerStatsData,
//     cpu_temp: Option<f64>,
//     network: HashMap<String, MoonrakerNetworkInterfaceStatsData>,
//     system_cpu_usage: HashMap<String, f32>,
//     websocket_connections: i64,
// }
//
// /// The toolhead object reports state of the current tool
// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct ToolheadData {
//     /// The axes that are homed
//     homed_axes: Vec<char>,
//     /// Internal value, not generally useful to clients
//     print_time: f32,
//     /// Internal value, not generally useful to clients
//     estimated_print_time: f32,
//     /// The name of the currently selected extruder
//     extruder: String,
//     /// The last position the tool was commanded to move, including any offsets added to an axis
//     position: XYZE,
//     /// The currently set maximum velocity
//     max_velocity: f32,
//     /// The currently set maximum acceleration
//     max_accel: f32,
//     /// The currently set maximum acceleration to deceleration
//     max_accel_to_decel: f32,
//     /// The currently set square corner velocity
//     square_corner_velocity: f32,
// }
