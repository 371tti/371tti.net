use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use bytes::Bytes;

use super::init::AppConfig;
use crate::handler::err_page::ErrHandler;

pub struct AppSet {
    pub app_config: AppConfig,
    pub err_handler: ErrHandler,
    pub static_cache: HashMap<String, Bytes>,
}

impl AppSet {
    pub async  fn new(app_config: AppConfig) -> Self {

        let static_dir = PathBuf::from("./static");
        let static_cache = Self::cache_static_files(&static_dir);
        let app_set = AppSet {
            app_config: app_config.clone(),
            err_handler: ErrHandler::new(app_config.template.clone()).await,
            static_cache: static_cache
        };
        app_set
    }

    fn cache_static_files(dir: &Path) -> HashMap<String, Bytes> {
        let mut cache = HashMap::new();
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if let Ok(content) = fs::read(&path) {
                                cache.insert(filename.to_string(), Bytes::from(content));
                            }
                        }
                    }
                }
            }
        }

        cache
    }
}
