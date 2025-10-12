pub mod err;
pub mod search;

pub struct PageGenerator {
    pub err_page: err::ErrPage,
    pub search_page: search::SearchPage,
}

impl PageGenerator {
    pub fn new() -> Self {
        Self {
            err_page: err::ErrPage::new(),
            search_page: search::SearchPage::new("http://localhost:90"),
        }
    }
}