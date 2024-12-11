use std::sync::Arc;

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};

use crate::{config::ServerConfig, share::{self, collection::Collection}};

pub struct IndexServer {
    pub config: ServerConfig,
    pub share: Arc<Collection>,
}

impl IndexServer {
    pub fn new(config: ServerConfig, share: Arc<Collection>) -> Self {
        Self {
            config: config,
            share: share,
        }
    }
    
    pub fn create_server(&self) -> Result<Server, std::io::Error> {
        let server = HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                // .wrap(middleware::ErrorHandlers::new().default_handler(err_handler))
                // .app_data(app_set.clone())
        })
        .bind(self.config.server_bind.clone())?
        .workers(self.config.server_workers.clone())
        .backlog(self.config.server_backlog.clone())
        .server_hostname("index")
        .run();

        Ok(server)
    }
}