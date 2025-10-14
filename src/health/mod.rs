use std::{collections::VecDeque, sync::RwLock};

use chrono::{DateTime, Utc};

pub struct HealthCheck {
    pub status: RwLock<VecDeque<HealthStatus>>,
}

pub enum HealthStatus {
    Ok {
        message: String,
        access_count: u64,
        latency: u64,
        timestamp: DateTime<Utc>,
    },
    SlowDown {
        message: String,
        access_count: u64,
        latency: u64,
        timestamp: DateTime<Utc>,
    },
    Down {
        message: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    Maintenance {
        message: String,
        timestamp: DateTime<Utc>,
    },
    None,
}