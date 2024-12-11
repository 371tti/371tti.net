use std::sync::Arc;

use crate::{config::{self, Configuration}, err_handler};

#[derive(Clone)]
pub struct Collection {
    pub err_handler: err_handler::handler::Handler,
    pub config: config::Configuration,
}

impl Collection {
    pub fn new(config: Configuration) -> Arc<Self> {
        let collection = Self {
            err_handler: err_handler::handler::Handler {},
            config: config,
        };

        Arc::new(collection)
    }
}