use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::Utc;

use crate::{
    TASK_SCHEDULER_WORKER_COUNT,
    config::Config,
    scheduler::{TaskID, TaskPriority, TaskScheduler, task},
    web::{
        TemplateService,
        analyzer::Counter,
        api::{DocsRouter, LsAPI, LsResponse},
    },
};

#[derive(Clone)]
pub struct SiteContext {
    pub shared: Arc<SiteContextShared>,
}

pub struct SiteContextShared {
    pub ls_api: LsAPI,
    pub docs_router: DocsRouter,
    pub counter: Counter,
    pub config: Config,
    pub scheduler: TaskScheduler,
    pub system_info: ArcSwap<SystemInfo>,
}

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
        let config = Config::load_or_create()?;
        let shared: Arc<SiteContextShared> = Arc::new(SiteContextShared {
            ls_api: LsAPI::new(&config.base_dir),
            docs_router: DocsRouter::new(&config.base_dir),
            counter: Counter::new(),
            config,
            scheduler: TaskScheduler::new(),
            system_info: ArcSwap::new(Arc::new(SystemInfo {
                system_version: crate::VERSION.to_string(),
                content_hash: "unknown".to_string(),
            })),
        });
        TaskScheduler::start(shared.clone(), TASK_SCHEDULER_WORKER_COUNT).await;
        shared
            .scheduler
            .push_task(
                TaskID::CRON,
                task::cron_task(),
                TaskPriority::HIGH,
                Some(Utc::now()),
                None,
            )
            .await;
        Ok(Self { shared })
    }

    pub async fn docs_routing(&self, path: &[&str]) -> std::io::Result<Option<String>> {
        self.shared
            .docs_router
            .route(path, self.shared.system_info.load_full().as_ref())
            .await
    }

    pub fn not_found_routing(&self) -> String {
        TemplateService::render_temp_html(
            include_str!("../../data/404.html").to_string(),
            self.shared.system_info.load_full().as_ref(),
        )
    }

    pub async fn ls_routing(&self, path: &[&str]) -> std::io::Result<LsResponse> {
        self.shared.ls_api.list(path).await
    }
}
