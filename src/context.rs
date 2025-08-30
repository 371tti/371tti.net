use std::sync::Arc;

use kurosabi::{request::Req, response::Res};

use crate::page_generator::PageGenerator;

#[derive(Clone)]
pub struct SiteContext {
    pub ssr: Arc<PageGenerator>,
}

impl SiteContext {
    pub fn new() -> Self {
        let ssr = Arc::new(PageGenerator::new()); // Assuming PageGenerator has a new() method
        Self { ssr }
    }

}