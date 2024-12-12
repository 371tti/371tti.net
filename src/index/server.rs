use std::{sync::Arc, thread::sleep};

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};
use actix_files as fs;
use std::time::Duration;
use log::{error, info};

use crate::{config::ServerConfig, server::server_trait::WkServer, share::{self, collection::Collection}, utils};

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
        let server = HttpServer::new(move || {
            let custom_logger = Logger::new(
                format!(
                    "\n[{}] \n\
                    Client IP: %a\n\
                    Request Line: \"%r\"\n\
                    Status Code: %s\n\
                    Response Size: %b bytes\n\
                    Referer: \"%{{Referer}}i\"\n\
                    User-Agent: \"%{{User-Agent}}i\"\n\
                    Processing Time: ps:%Tms\n\
                    Send Time: send:%Dms",
                    server_name // サーバー名（例: "IndexServer"）
                )
                .as_str(),
            );
            App::new()
                .wrap(custom_logger)
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