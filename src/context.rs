use crate::page_generator::PageGenerator;

pub struct SiteContext {
    pub ssr: PageGenerator,
}

impl SiteContext {
    pub fn new() -> Self {
        let ssr = PageGenerator::new(); // Assuming PageGenerator has a new() method
        Self { ssr }
    }

}