use std::sync::Arc;

use kurosabi::{request::Req, response::Res};

use crate::{page_generator::PageGenerator, user_manager::auth_manager::AuthManager};

#[derive(Clone)]
pub struct SiteContext {
    pub ssr: Arc<PageGenerator>,
    pub aurth_manager: Arc<AuthManager>,
    pub user: Option<String>, // Assuming user is a String, change as needed
    pub session_id: String,
}

impl SiteContext {
    pub fn new() -> Self {
        let ssr = Arc::new(PageGenerator::new()); // Assuming PageGenerator has a new() method
        let aurth_manager = Arc::new(AuthManager::new());
        let user = None;
        let session_id = String::new(); // Initialize with an empty string or a default value
        Self { ssr, aurth_manager, user, session_id }
    }
    
    pub fn init(&mut self, req: &mut Req, res: &mut Res) {
        let (user, session_id) = self.aurth_manager.check_session(req, res);
        self.user = user;
        self.session_id = session_id;
    }
}