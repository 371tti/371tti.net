use std::{sync::Arc, thread::sleep};

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};
use std::time::Duration;
use log::{error, info};

use crate::{config::ServerConfig, server::server_trait::WkServer, share::{self, collection::Collection}};

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
                // 他のミドルウェアやデータをここに追加可能
        })
        .bind(self.config.server_bind.clone())?
        .workers(self.config.server_workers)
        .backlog(self.config.server_backlog)
        .run();

        Ok(server)
    }
}

impl WkServer for IndexServer {
    fn config(&self) -> &ServerConfig {
        &self.config
    }

    fn create_server(&self) -> Result<Server, std::io::Error> {
        IndexServer::create_server(self)
    }

    fn server_name(&self) -> &str {
        "IndexServer"
    }

    fn failed_report(&mut self, e: std::io::Error, failure_count: u32, start_time: tokio::time::Instant) {
        
    }
    
}