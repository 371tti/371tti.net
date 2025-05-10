pub mod err_page;

pub struct PageGenerator {
    pub err_page: err_page::ErrPage,
}

impl PageGenerator {
    pub fn new() -> Self {
        Self {
            err_page: err_page::ErrPage::new(),
        }
    }
}