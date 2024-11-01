use super::init::AppConfig;
use crate::handler::err_page::ErrHandler;

pub struct AppSet {
    pub app_config: AppConfig,
    pub err_handler: ErrHandler,
}

impl AppSet {
    pub async  fn new(app_config: AppConfig) -> Self {
        let app_set = AppSet {
            app_config: app_config.clone(),
            err_handler: ErrHandler::new(app_config.template.clone()).await,
        };
        app_set
    }
}
