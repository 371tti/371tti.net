use std::{sync::Arc, thread::sleep, time::Duration};

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};
use log::{error, info};

use crate::{config::ServerConfig, server::server_trait::WkServer, share::{self, collection::Collection}};

pub struct TransferServer {
    pub config: ServerConfig,
    pub share: Arc<Collection>,
}

impl TransferServer {
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
        .run();

        Ok(server)
    }

}

impl WkServer for TransferServer {
    fn config(&self) -> &ServerConfig {
        &self.config
    }

    fn create_server(&self) -> Result<Server, std::io::Error> {
        TransferServer::create_server(self)
    }

    fn server_name(&self) -> &str {
        "TransferServer"
    }

    fn failed_report(&mut self, e: std::io::Error, failure_count: u32, start_time: tokio::time::Instant) {
        
    }
    
}