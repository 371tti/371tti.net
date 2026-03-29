use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::Utc;
use kurosabi::http::header::{Cookie, CookieBuilder};
use srv_session::{
    AuthManager,
    serde_hex_array::{bytes_to_hex, hex_to_bytes},
};

use crate::{
    SESSION_COOKIE_NAME, TASK_SCHEDULER_WORKER_COUNT,
    config::Config,
    file::FileService,
    index::index::Index,
    scheduler::{TaskID, TaskPriority, TaskScheduler, task},
    state::{AccountKV, SessionKV, Storage},
    web::{TemplateService, api::DocsRouter},
};

#[derive(Clone)]
pub struct SiteContext {
    pub shared: Arc<SiteContextShared>,
    pub now_user: Option<Box<str>>,
    pub new_session: bool,
}

pub struct SiteContextShared {
    pub docs_router: DocsRouter,
    pub config: Arc<Config>,
    pub scheduler: TaskScheduler,
    pub system_info: ArcSwap<SystemInfo>,
    pub storage: Storage,
    pub auth_manager: AuthManager<SessionKV, AccountKV>,
    pub index: Index,
    pub file_service: FileService,
}

#[derive(Clone)]
pub struct SystemInfo {
    pub system_version: String,
    pub content_hash: String,
}

impl SystemInfo {
    pub fn text(&self) -> String {
        format!(
            "{}+contents.git.{}",
            self.system_version,
            &self.content_hash[..7]
        )
    }
}

impl SiteContext {
    pub async fn new() -> std::io::Result<Self> {
        let config = Arc::new(Config::load_or_create()?);
        let storage = Storage::load_or_create(config.clone())?;
        let auth_manager = AuthManager::new(
            storage.sessions.clone(),
            storage.accounts.clone(),
            config.http_config.session_timeout,
            config.http_config.account_timeout,
            config.hash_config.clone(),
        );
        let index = Index::new(config.clone()).await;
        let file_service = FileService::new(config.clone());
        let shared: Arc<SiteContextShared> = Arc::new(SiteContextShared {
            docs_router: DocsRouter::new(config.clone()),
            config,
            scheduler: TaskScheduler::new(),
            system_info: ArcSwap::new(Arc::new(SystemInfo {
                system_version: crate::VERSION.to_string(),
                content_hash: "unknown".to_string(),
            })),
            storage,
            auth_manager,
            index,
            file_service,
        });
        // Start the scheduler and push the cron task
        TaskScheduler::start(shared.clone(), TASK_SCHEDULER_WORKER_COUNT).await;
        shared
            .scheduler
            .push_task(
                TaskID::INIT,
                task::init_task(),
                TaskPriority::HIGH,
                Some(Utc::now()),
                None,
            )
            .await;
        Ok(Self {
            shared,
            now_user: None,
            new_session: true,
        })
    }

    pub async fn docs_routing(&self, path: &[&str]) -> std::io::Result<Option<String>> {
        self.shared.docs_router.route(path, &self.shared).await
    }

    pub fn not_found_routing(&self) -> String {
        TemplateService::render_temp_html(
            include_str!("../../static/404.html").to_string(),
            &self.shared,
        )
    }

    pub fn session_check(&mut self, hex_session: Option<&str>) -> Option<Cookie> {
        if let Some(hex) = hex_session {
            if let Ok(session_bin) = hex_to_bytes(hex) {
                if let Some(v) = self
                    .shared
                    .auth_manager
                    .get_and_verify_session(&session_bin)
                {
                    self.now_user = v.primary_account;
                    self.new_session = false;
                    return None;
                }
            }
        }
        let new_session_bin = self.shared.auth_manager.create_session();
        let hex_session_id = bytes_to_hex(&new_session_bin);
        self.now_user = None;
        self.new_session = true;
        Some(
            CookieBuilder::new(SESSION_COOKIE_NAME, hex_session_id)
                .path("/")
                .http_only(true)
                .secure(true)
                .max_age(self.shared.config.http_config.cookie_max_age_seconds)
                .build(),
        )
    }
}
