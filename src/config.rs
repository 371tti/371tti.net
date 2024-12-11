use std::{fs, path::PathBuf};

use actix_web::middleware::Logger;
use serde::Deserialize;
use log::{error, info};

use crate::{index::loader, utils};

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub enable: bool,
    pub server_bind: String,
    pub server_workers: usize,
    pub server_backlog: u32,
    pub restart_on_panic: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    pub index_server: ServerConfig,
    pub transfer_server: ServerConfig,
    pub path: String,
    pub logger_mode: String,
}

impl Configuration {
    pub fn loader(yaml_path_r: &str) -> Self {
        // バイナリのディレクトリから相対パスでファイルを指定
        let config_path = match utils::fs::get_file_path(yaml_path_r) {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to get config file path: {}", e);
                panic!("Failed to get config file path");
            }
            
        };
        let yaml_string = if let Ok(content) = fs::read_to_string(config_path) {
            content
        } else {
            error!("Failed to read config file");
            panic!("Failed to read config file");
        };


        let config = match serde_yaml::from_str::<Configuration>(&yaml_string) {
            Ok(config) => config,
            Err(e) => {
                match e.location() {
                    Some(location) => {
                        error!("Syntax error in YAML file at line {}, column {}: {}",
                            location.line(),
                            location.column(),
                            e
                    );
                    }
                    None => {
                        error!("Failed to parse config file: {}", e);
                    }
                    
                }
                panic!("Failed to parse config file");
            }
            
        };
        info!("Config file loaded successfully");
        config
    }
}