use kurosabi::kurosabi::Context;

use crate::context::SiteContext;

pub struct StatusPage;

impl StatusPage {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn page(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.res.html(include_str!("../../data/pages/status/index.html"));
        c
    }
}