pub mod config;
pub mod markdown;
pub mod render;
pub mod scheduler;
pub mod updater;
pub mod web;
pub mod state;

pub const TASK_SCHEDULER_WORKER_COUNT: usize = 4;

pub const CONFIG_FILE_NAME: &str = "config.yaml";
pub const STORAGE_FILE_NAME: &str = "storage.cbor";

pub const DEFAULT_BASE_DIR: &str = "./data/";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const AUTH_HASH_TARGET_MS: u64 = 200;

pub const DEFAULT_SESSION_TIMEOUT_HOURS: i64 = 24 * 30;
pub const DEFAULT_ACCOUNT_TIMEOUT_HOURS: i64 = 24 * 7;
pub const DEFAULT_COOKIE_MAX_AGE_SECONDS: u64 = 60 * 60 * 24 * 30;

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub const LOGO_AA: &str = r#"
█ █ █ █     ██  ███  ██ ███ ███ ███       █ ███ ███
███ ██  ███  ██ █ █   █  █   █   █  ███ ███ ██   █
███ █ █     ██    █   █  █   █  ███     █   ███  █   "#;

pub fn print_logo() {
    log::info!("\n\n{}v{} Starting up...\n\nLicense: MIT\n", LOGO_AA, VERSION);
}