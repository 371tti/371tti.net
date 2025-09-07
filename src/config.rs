use std::path::PathBuf;

use log::info;
use serde::{Deserialize, Serialize};

use crate::user_manager::hash_config::HashConfig;

#[derive(Serialize, Deserialize)]
pub struct MainConfig {
    /// session timeout in seconds
    /// min 0, Max u64::MAX
    pub session_timeout: u64,
    pub hash_config: HashConfig,
}

impl MainConfig {
    pub fn save_to_file(&self, path: &PathBuf) {
        let toml_str = toml::to_string_pretty(self).expect("Failed to serialize config to TOML");
        std::fs::write(path, toml_str).expect("Failed to write config to file");
    }

    /// Load configuration from a file
    /// If the file does not exist, create a new one with default settings and exit the program.
    pub fn load_from_file(path: &PathBuf) -> Self {
        let toml_str = std::fs::read_to_string(path).map(|s| s).unwrap_or_else(|_| {
            info!("Config file not found or unreadable.");
            Self::initialize_new(path.clone());
            String::new() // This line will never be reached
        });
        toml::from_str(&toml_str).expect("Failed to deserialize config from TOML")
    }

    /// Create a new config file with default settings and exit the program.
    fn initialize_new(path: PathBuf) {
        info!("No config file found, creating a new one at {:?}", path);
        // create default config
        // with benchmarked hash config
        let hash_config = HashConfig::benchmark();
        let config = Self {
            session_timeout: 604800,
            hash_config,
        };
        // save to file
        config.save_to_file(&path);
        info!("Default config file created. Please edit it and restart the server.");
        std::process::exit(0);
    }
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            // 1 week
            session_timeout: 604800,
            // after set by benchmark
            hash_config: HashConfig {
                pepper: "default_pepper".to_string(), // Placeholder, will be set by benchmark
                memory_kib: 65536, // 64 MiB
                time_cost: 3,
                lanes: 4,
            },
        }
    }
}