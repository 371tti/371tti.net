use std::io::Error;
use std::sync::Arc;

use actix_web::cookie::time::error;
use actix_web::dev::ServiceResponse;
use config::{Configuration};
use share::collection::{self, Collection};
use tokio;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::{ErrorHandlerResponse, Logger};
use env_logger::Env;

use log::{error, info};

mod index;
mod config;
mod share;
mod err_handler;
mod utils;
mod transfer;

async fn server_start(config: Configuration, collection: Arc<Collection>) -> Result<(), Error> {
    let index_server = index::server::IndexServer::new(config.index_server, Arc::clone(&collection)).create_server().unwrap();
    let transfer_server = transfer::server::TransferServer::new(config.transfer_server, Arc::clone(&collection)).create_server().unwrap();
    // 追加していくの

    let result = tokio::join!(
        index_server,
        transfer_server,
    );

    if let Err(e) = result.0 {
        error!("Failed to start index server: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = result.1 {
        error!("Failed to start redirector server: {}", e);
        std::process::exit(1);
    }



    result.0
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let config = config::Configuration::loader("config.yaml");
    let collection = collection::Collection::new(config.clone());

    server_start(config, collection).await?;
    Ok(())
}