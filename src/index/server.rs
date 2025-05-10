use std::sync::Arc;

use actix_web::{dev::Server, middleware::{self, Logger}, web, App, HttpServer};
use actix_files as fs;
use actix_cors::Cors;

use log::{error, info};

use crate::{actix_middleware::status_page, config::ServerConfig, server::server_trait::WkServer, share::{self, collection::Collection}, utils};

use super::config::ServiceConfig;

pub struct IndexServer {
    pub config: ServerConfig<ServiceConfig>,
    pub share: Arc<Collection>,
}

impl IndexServer {
    pub fn new(config: ServerConfig<ServiceConfig>, share: Arc<Collection>) -> Self {
        Self {
            config: config,
            share: share,
        }
    }
    
    pub fn create_server(&self) -> Result<Server, std::io::Error> {
        let path = match utils::fs::get_file_path(&self.share.config.path) {
            Ok(p) => p.join(self.share.config.index_server.service_config.path.clone()),
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        };

        let server_name = self.server_name().to_string();
        let path_clone = path.clone();
        let share_clone = web::Data::new(Arc::clone(&self.share));
        let server = HttpServer::new(move || {
            let custom_logger = utils::logger::custom_actix_logger(&server_name);
            App::new()
                .app_data(share_clone.clone())
                .wrap(custom_logger)
                .wrap(Cors::permissive())
                .wrap(middleware::ErrorHandlers::new().default_handler(status_page::middleware::Handler::err_handler))
                // 他のミドルウェアやデータをここに追加可能
                .service(fs::Files::new("/", path_clone.clone()).index_file("index.html"))
        })
        .bind(self.config.server_bind.clone())?
        .workers(self.config.server_workers)
        .backlog(self.config.server_backlog)
        .run();

        Ok(server)
    }   
}

impl WkServer<ServiceConfig> for IndexServer {
    fn config(&self) -> &ServerConfig<ServiceConfig> {
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