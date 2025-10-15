use std::{collections::VecDeque, sync::{atomic::{AtomicU64, Ordering}, RwLock}};

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{ser::SerializeStruct, Serialize};

use crate::config::MainConfig;

const SELF_SLOW_DOWN_THRESHOLD_MS: u64 = 600;
const SEARCH_SLOW_DOWN_THRESHOLD_MS: u64 = 1000;

pub struct HealthChecker {
    pub wk: RwLock<VecDeque<HealthStatus>>,
    pub wl_search_engine: RwLock<VecDeque<HealthStatus>>,
    pub uptime: DateTime<Utc>,
    pub access_counter: AtomicU64,
    pub search_access_counter: AtomicU64,
    // 以下シリアライズ未対象
    client: Client,
    pub self_url: String,
    pub search_url: String,
}

impl Serialize for HealthChecker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let wk = self.wk.read().unwrap();
        let wl_search_engine = self.wl_search_engine.read().unwrap();
        let mut state = serializer.serialize_struct("HealthChecker", 6)?;
        state.serialize_field("wk", &*wk)?;
        state.serialize_field("wl_search_engine", &*wl_search_engine)?;
        state.serialize_field("uptime", &self.uptime)?;
        state.serialize_field("access_counter", &self.access_counter.load(Ordering::Relaxed))?;
        state.serialize_field("search_access_counter", &self.search_access_counter.load(Ordering::Relaxed))?;
        state.end()
    }
}

#[derive(Clone, Serialize)]
pub enum HealthStatus {
    Ok {
        message: String,
        latency: u64,
        timestamp: DateTime<Utc>,
        access_count: u64,
    },
    SlowDown {
        message: String,
        latency: u64,
        timestamp: DateTime<Utc>,
        access_count: u64,
    },
    Down {
        message: String,
        error: String,
        timestamp: DateTime<Utc>,
        access_count: u64,
    },
    Error {
        message: String,
        error: String,
        timestamp: DateTime<Utc>,
        access_count: u64,
    },
    Maintenance {
        message: String,
        timestamp: DateTime<Utc>,
        access_count: u64,
    },
    None,
}

impl HealthChecker {
    pub fn new(config: &MainConfig) -> Self {
        let client = Client::builder()
            .tcp_keepalive(None)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("build reqwest client");
        let self_url = format!("https://{}/ping", config.domain);
        let search_url = format!("{}/ping", config.api_endpoints.search);
        HealthChecker {
            wk: RwLock::new(VecDeque::with_capacity(288)),
            wl_search_engine: RwLock::new(VecDeque::with_capacity(288)),
            client,
            self_url,
            search_url,
            uptime: Utc::now(),
            access_counter: AtomicU64::new(0),
            search_access_counter: AtomicU64::new(0),
        }
    }

    pub async fn update(&self) {
        let self_status = self.check_self(&self.self_url, SELF_SLOW_DOWN_THRESHOLD_MS).await;
        let search_status = self.check_wl_search_engine(&self.search_url, SEARCH_SLOW_DOWN_THRESHOLD_MS).await;

        {
            let mut wk = self.wk.write().unwrap();
            if wk.len() == 288 {
                wk.pop_front();
            }
            wk.push_back(self_status);
        }

        {
            let mut wl_search_engine = self.wl_search_engine.write().unwrap();
            if wl_search_engine.len() == 288 {
                wl_search_engine.pop_front();
            }
            wl_search_engine.push_back(search_status);
        }
    }

    pub fn add_count(&self) {
        self.access_counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_search_count(&self) {
        self.search_access_counter.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn check_self(&self, url: &str, slow_down_threshold: u64) -> HealthStatus {
        let start = Utc::now();
        let res = self.client.get(url).send().await;
        match res {
            Ok(resp) => {
                let latency = (Utc::now() - start).num_milliseconds() as u64;
                if resp.status().is_success() {
                    if latency > slow_down_threshold {
                        HealthStatus::SlowDown {
                            message: format!("Service is slow: {} ms", latency),
                            latency,
                            timestamp: Utc::now(),
                            access_count: self.access_counter.load(Ordering::Relaxed),
                        }
                    } else {
                        HealthStatus::Ok {
                            message: "Service is healthy".to_string(),
                            latency,
                            timestamp: Utc::now(),
                            access_count: self.access_counter.load(Ordering::Relaxed),
                        }
                    }
                } else {
                    HealthStatus::Error {
                        message: format!("Service returned error status: {}", resp.status()),
                        error: resp.status().to_string(),
                        timestamp: Utc::now(),
                        access_count: self.access_counter.load(Ordering::Relaxed),
                    }
                }
            }
            Err(e) => HealthStatus::Down {
                message: "Disconnected Network".to_string(),
                error: e.to_string(),
                timestamp: Utc::now(),
                access_count: self.access_counter.load(Ordering::Relaxed),
            },
        }
    }

    pub async fn check_wl_search_engine(&self, url: &str, slow_down_threshold: u64) -> HealthStatus {
        let start = Utc::now();
        let res = self.client.get(url).send().await;
        match res {
            Ok(resp) => {
                let latency = (Utc::now() - start).num_milliseconds() as u64;
                if resp.status().is_success() {
                    if latency > slow_down_threshold {
                        HealthStatus::SlowDown {
                            message: format!("Search engine is slow: {} ms", latency),
                            latency,
                            timestamp: Utc::now(),
                            access_count: self.search_access_counter.load(Ordering::Relaxed),
                        }
                    } else {
                        HealthStatus::Ok {
                            message: "Search engine is healthy".to_string(),
                            latency,
                            timestamp: Utc::now(),
                            access_count: self.search_access_counter.load(Ordering::Relaxed),
                        }
                    }
                } else {
                    HealthStatus::Error {
                        message: format!("Search engine returned error status: {}", resp.status()),
                        error: resp.status().to_string(),
                        timestamp: Utc::now(),
                        access_count: self.search_access_counter.load(Ordering::Relaxed),
                    }
                }
            }
            Err(e) => HealthStatus::Down {
                message: "Disconnected Network".to_string(),
                error: e.to_string(),
                timestamp: Utc::now(),
                access_count: self.search_access_counter.load(Ordering::Relaxed),
            },
        }
    }
}