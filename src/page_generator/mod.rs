pub mod err_page;
pub mod search_page;

pub struct PageGenerator {
    pub err_page: err_page::ErrPage,
    pub search_page: search_page::SearchPage,
}

impl PageGenerator {
    pub fn new() -> Self {
        Self {
            err_page: err_page::ErrPage::new(),
            search_page: search_page::SearchPage::new("http://localhost:90"),
        }
    }
}