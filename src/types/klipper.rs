use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct McuStats {
    bytes_invalid: u64,
    bytes_read: u64,
    bytes_retransmit: u64,
    bytes_write: u64,
    freq: u64,
    mcu_awake: f64,
    mcu_task_avg: f64,
    ready_bytes: u64,
    upcoming_bytes: u64,
    send_seq: u64,
    receive_seq: u64,
    retransmit_seq: u64,
    rto: f64,
    rttvar: f64,
}
