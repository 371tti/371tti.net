use std::sync::Arc;

use crate::{
    config::BASE_DIR,
    web::{analyzer::Counter, api::{DocsRouter, LsAPI, LsResponse}, templates::TemplateService},
};

#[derive(Clone)]
pub struct SiteContext {
    pub shared: Arc<SiteContextShared>,
}

pub struct SiteContextShared {
    pub ls_api: LsAPI,
    pub docs_router: DocsRouter,
    pub counter: Counter,
}

impl Default for SiteContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SiteContext {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(SiteContextShared {
                ls_api: LsAPI::new(BASE_DIR),
                docs_router: DocsRouter::new(
                    BASE_DIR,
                    TemplateService::default(),
                ),
                counter: Counter::new(),
            })
        }
    }

    pub async fn docs_routing(&self, path: &[&str]) -> std::io::Result<Option<String>> {
        self.shared.docs_router.route(path).await
    }

    pub async fn ls_routing(&self, path: &[&str]) -> std::io::Result<LsResponse> {
        self.shared.ls_api.list(path).await
    }

}

