use std::{sync::Arc, thread::sleep, time::Duration};

use actix_web::{dev::Server, middleware::{self, Logger}, App, HttpServer};
use log::{error, info};

use crate::{config::ServerConfig, share::{self, collection::Collection}};

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
        .server_hostname("transfer")
        .run();

        Ok(server)
    }

/// サーバーを開始し、再起動の管理を行う
pub async fn run_with_restart(self) -> Result<(), std::io::Error> {
    if !self.config.enable {
        info!("TransferServer is disabled.");
        return Ok(());
    }

    loop {
        match self.create_server() {
            Ok(server) => {
                if let Err(e) = server.await {
                    error!("TransferServer encountered an error: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to initialize TransferServer: {}", e);
            }
        }

        if !self.config.restart_on_panic {
            error!("TransferServer is set to not restart on panic.");
            break;
        }

        // 再起動前に少し待機
        error!("Restarting TransferServer in 5 seconds...");
        sleep(Duration::from_secs(5));
    }

    Ok(())
}

}