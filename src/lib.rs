pub mod config;
pub mod markdown;
pub mod render;
pub mod scheduler;
pub mod updater;
pub mod web;

pub const TASK_SCHEDULER_WORKER_COUNT: usize = 4;

pub const CONFIG_FILE_NAME: &str = "config.yaml";

pub const DEFAULT_BASE_DIR: &str = "./data/";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
