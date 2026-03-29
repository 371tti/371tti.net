use std::{path::Path, time::Duration};

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use srv_session::HashConfig;

use crate::{
    AUTH_HASH_TARGET_MS, DEFAULT_ACCOUNT_TIMEOUT_HOURS, DEFAULT_BASE_DIR,
    DEFAULT_COOKIE_MAX_AGE_SECONDS, DEFAULT_SESSION_TIMEOUT_HOURS, STORAGE_FILE_NAME,
    utils::byte_size_serde, 
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub base_dir: String,
    pub storage_config: StorageConfig,
    pub http_config: HttpConfig,
    pub git_config: GitConfig,
    pub cache_config: CacheConfig,
    pub hash_config: HashConfig,
}

#[derive(Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(with = "byte_size_serde")]
    pub max_memory_bytes: u64,
    #[serde(with = "byte_size_serde")]
    pub max_entry_size_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct StorageConfig {
    pub storage_file: String,
}

#[derive(Serialize, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    #[serde(with = "humantime_serde")]
    pub session_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub account_timeout: Duration,
    pub cookie_max_age_seconds: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GitConfig {
    pub enable_remote: bool,
    pub remote_url: String,
    pub remote_branch: String,
    pub user: Option<String>,
    pub token: Option<String>,
}

impl GitConfig {
    pub fn auth_able_url(&self) -> String {
        if let Some(token) = &self.token {
            self.remote_url.replace(
                "https://",
                &format!(
                    "https://{}:{}@",
                    self.user.as_deref().unwrap_or("x-access-token"),
                    token
                ),
            )
        } else {
            self.remote_url.clone()
        }
    }

    pub fn refname(&self) -> String {
        format!("refs/heads/{}", self.remote_branch)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_config: StorageConfig {
                storage_file: STORAGE_FILE_NAME.to_string(),
            },
            base_dir: DEFAULT_BASE_DIR.to_string(),
            http_config: HttpConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                session_timeout: Duration::from_hours(DEFAULT_SESSION_TIMEOUT_HOURS),
                account_timeout: Duration::from_hours(DEFAULT_ACCOUNT_TIMEOUT_HOURS),
                cookie_max_age_seconds: DEFAULT_COOKIE_MAX_AGE_SECONDS,
            },
            git_config: GitConfig {
                enable_remote: true,
                remote_url: "https://github.com/371tti/371tti.net-contents".to_string(),
                remote_branch: "main".to_string(),
                user: None,
                token: None,
            },
            cache_config: CacheConfig {
                max_memory_bytes: crate::DEFAULT_MAX_CACHE_MEMORY_BYTES,
                max_entry_size_bytes: crate::DEFAULT_MAX_CACHE_ENTRY_SIZE_BYTES,
            },
            hash_config: HashConfig::benchmark(AUTH_HASH_TARGET_MS),
        }
    }
}

impl Config {
    pub fn get_host(&self) -> [u8; 4] {
        let segments: Vec<&str> = self.http_config.host.split('.').collect();
        if segments.len() != 4 {
            error!(
                "Invalid host format in config: '{}', defaulting to '0.0.0.0'",
                self.http_config.host
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
