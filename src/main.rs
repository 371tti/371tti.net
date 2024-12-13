use std::io::Error;
use std::sync::Arc;

use actix_web::cookie::time::error;
use actix_web::dev::ServiceResponse;
use config::{Configuration};
use server::server_trait::WkServer;
use share::collection::{self, Collection};
use tokio;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::{ErrorHandlerResponse, Logger};
use env_logger::Env;

use log::{error, info};

mod index;
mod config;
mod share;
mod actix_middleware;
mod utils;
mod transfer;
mod server;

async fn server_start(config: Configuration, collection: Arc<Collection>) -> Result<(), Error> {
    let index_server = index::server::IndexServer::new(config.index_server, Arc::clone(&collection)).run_with_restart();
    let transfer_server = transfer::server::TransferServer::new(config.transfer_server, Arc::clone(&collection)).run_with_restart();
    // 追加していくの

    let result = tokio::join!(
        index_server,
        transfer_server,
    );

    match result {
        (Ok(_), Ok(_)) => {
            info!("All servers have stopped.");
            Ok(())
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("An error occurred: {}", e);
            Err(e)
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let config = config::Configuration::loader("config.yaml");
    let collection = collection::Collection::new(config.clone());

    server_start(config, collection).await?;
    Ok(())
}