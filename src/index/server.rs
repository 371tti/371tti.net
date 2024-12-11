use std::{sync::Arc, thread::sleep};

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};
use std::time::Duration;
use log::{error, info};

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
                // 他のミドルウェアやデータをここに追加可能
        })
        .bind(self.config.server_bind.clone())?
        .workers(self.config.server_workers)
        .backlog(self.config.server_backlog)
        .server_hostname("index")
        .run();

        Ok(server)
    }

    /// サーバーを開始し、再起動の管理を行う
    pub async fn run_with_restart(self) -> Result<(), std::io::Error> {
        if !self.config.enable {
            info!("IndexServer is disabled.");
            return Ok(());
        }

        loop {
            match self.create_server() {
                Ok(server) => {
                    if let Err(e) = server.await {
                        error!("IndexServer encountered an error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to initialize IndexServer: {}", e);
                }
            }

            if !self.config.restart_on_panic {
                error!("IndexServer is set to not restart on panic.");
                break;
            }

            // 再起動前に少し待機
            error!("Restarting IndexServer in 5 seconds...");
            sleep(Duration::from_secs(5));
        }

        Ok(())
    }
}