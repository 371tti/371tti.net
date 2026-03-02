use std::path::Path;

use chrono::Duration;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use srv_session::HashConfig;

use crate::{AUTH_HASH_TARGET_MS, DEFAULT_ACCOUNT_TIMEOUT_HOURS, DEFAULT_BASE_DIR, DEFAULT_COOKIE_MAX_AGE_SECONDS, DEFAULT_SESSION_TIMEOUT_HOURS, STORAGE_FILE_NAME};

fn default_content_repo_branch() -> String {
    "main".to_string()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub storage_file: String,
    pub base_dir: String,
    pub port: u16,
    pub host: String,
    pub auto_content_update: bool,
    pub content_repo_url: String,
    #[serde(default = "default_content_repo_branch")]
    pub content_repo_branch: String,
    pub session_timeout: Duration,
    pub account_timeout: Duration,
    pub cookie_max_age_seconds: u64,
    pub hash_config: HashConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_file: STORAGE_FILE_NAME.to_string(),
            base_dir: DEFAULT_BASE_DIR.to_string(),
            port: 8080,
            host: "0.0.0.0".to_string(),
            auto_content_update: true,
            content_repo_url: "https://github.com/371tti/371tti.net-contents".to_string(),
            content_repo_branch: default_content_repo_branch(),
            session_timeout: Duration::hours(DEFAULT_SESSION_TIMEOUT_HOURS),
            account_timeout: Duration::hours(DEFAULT_ACCOUNT_TIMEOUT_HOURS),
            cookie_max_age_seconds: DEFAULT_COOKIE_MAX_AGE_SECONDS,
            hash_config: HashConfig::benchmark(AUTH_HASH_TARGET_MS)
        }
    }
}

impl Config {
    pub fn get_host(&self) -> [u8; 4] {
        let segments: Vec<&str> = self.host.split('.').collect();
        if segments.len() != 4 {
            error!(
                "Invalid host format in config: '{}', defaulting to '0.0.0.0'",
                self.host
            );
            return [0, 0, 0, 0];
        }
        let mut result = [0u8; 4];
        for (i, segment) in segments.iter().enumerate() {
            match segment.parse::<u8>() {
                Ok(val) => result[i] = val,
                Err(e) => {
                    error!(
                        "Invalid host segment '{}' in config: {}, defaulting to '0.0.0.0'",
                        segment, e
                    );
                    return [0, 0, 0, 0];
                }
            }
        }
        result
    }

    pub fn load_or_create() -> std::io::Result<Self> {
        let config_path = Path::new(crate::CONFIG_FILE_NAME);
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: Self = match serde_yaml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!(
                        "Failed to parse config file: {}, using default config. Error: {}",
                        crate::CONFIG_FILE_NAME,
                        e
                    );
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Failed to parse config file: {}, error: {}",
                            crate::CONFIG_FILE_NAME,
                            e
                        ),
                    ));
                }
            };
            info!(
                "Config loaded successfully from '{}'",
                crate::CONFIG_FILE_NAME
            );
            Ok(config)
        } else {
            let default_config = Self::default();
            let yaml = serde_yaml::to_string(&default_config).unwrap();
            std::fs::write(config_path, yaml)?;
            warn!(
                "Config file not found. A default config has been created at '{}'. Please review and modify it as needed.",
                crate::CONFIG_FILE_NAME
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Config file not found. A default config has been created at '{}'. Please review and modify it as needed.",
                    crate::CONFIG_FILE_NAME
                ),
            ))
        }
    }
}
