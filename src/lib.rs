pub mod config;
pub mod file;
pub mod git;
pub mod index;
pub mod markdown;
pub mod render;
pub mod scheduler;
pub mod state;
pub mod utils;
pub mod web;

pub const DOMAIN: &str = "371tti.net";

pub const TASK_SCHEDULER_WORKER_COUNT: usize = 4;

pub const CONFIG_FILE_NAME: &str = "config.yaml";
pub const STORAGE_FILE_NAME: &str = "storage.cbor";

pub const DEFAULT_BASE_DIR: &str = "./data/";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_FONT_DATA: &[u8] = include_bytes!("../static/UDEVGothicHSLG-Regular.ttf");
pub const DEFAULT_FONT_NAME: &str = "UDEV Gothic HSLG";

pub const AUTH_HASH_TARGET_MS: u64 = 200;

pub const DEFAULT_SESSION_TIMEOUT_HOURS: u64 = 24 * 30;
pub const DEFAULT_ACCOUNT_TIMEOUT_HOURS: u64 = 24 * 7;
pub const DEFAULT_COOKIE_MAX_AGE_SECONDS: u64 = 60 * 60 * 24 * 30;

pub const DEFAULT_MAX_CACHE_MEMORY_BYTES: u64 = 1024 * 1024 * 100; // 100MB
pub const DEFAULT_MAX_CACHE_ENTRY_SIZE_BYTES: u64 = 1024 * 1024 * 10; // 10MB

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub const LOGO_AA: &str = r#"
█ █ █ █     ██  ███  ██ ███ ███ ███       █ ███ ███
███ ██  ███  ██ █ █   █  █   █   █  ███ ███ ██   █
███ █ █     ██    █   █  █   █  ███     █   ███  █   "#;

pub fn print_logo() {
    log::info!(
        "\n\n{}v{} Starting up...\n\nLicense: MIT\n",
        LOGO_AA,
        VERSION
    );
}
