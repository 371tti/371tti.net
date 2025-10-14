use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum ServerStatusRes {
    #[serde(rename = "true")]
    Success {
        cpu_usage: Vec<f32>,
        ram_usage: Vec<f32>,
        access_log: Vec<u32>,
        uptime: DateTime<Utc>,
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
}