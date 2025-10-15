use kurosabi::kurosabi::Context;

use crate::{api::schema::status::ServerStatusRes, context::SiteContext};

pub struct StatusAPI;

impl StatusAPI {
    pub async fn ping(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.res.text("pong");
        c
    }

    pub async fn health(mut c: Context<SiteContext>) -> Context<SiteContext> {
        let status = ServerStatusRes::Success { 
            health: c.c.health.clone()
        };
        c.res.json_value(&serde_json::to_value(&status).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
        c
    }
}