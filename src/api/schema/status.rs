
use std::sync::Arc;

use serde::Serialize;

use crate::health::HealthChecker;


#[derive(Serialize)]
#[serde(tag = "success")]
pub enum ServerStatusRes {
    #[serde(rename = "true")]
    Success {
        health: Arc<HealthChecker>
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
}