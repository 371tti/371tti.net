use std::time::Duration;

use kurosabi::kurosabi::Context;
use log::error;

use crate::{api::schema::{self, search::IndexReq}, context::SiteContext};

pub struct SearchAPI;

impl SearchAPI {
    /// proxy to backend search API
    pub async fn search(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.c.health.add_search_count();
        let query = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        let result = c.c.ssr.search_page.search_api(query).await;

        match result {
            Ok(resp) => {
                c.res.json_value(&serde_json::to_value(&resp).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                c.res.set_status(200);
            }
            Err(code) => { c.res.set_status(code); }
        }
        c
    }

    /// proxy to backend index API
    pub async fn index(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.c.health.add_search_count();
        // Deserialize the request body into IndexReq
        let index_req = match c.req.body_de_struct::<IndexReq>().await {
            Ok(req) => req,
            Err(_e) => {
                c.res.text("Bad Request: Failed to parse request body");
                c.res.set_status(400);
                return c;
            }
        };

        // Proxy the request to the backend API
        let url = "http://127.0.0.1:90/add";

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        match client
            .post(url)
            .json(&index_req)
            .send()
            .await
        {
            Ok(resp) => {
                c.res.set_status(resp.status().as_u16());
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/json".to_string());
                match resp.bytes().await {
                    Ok(bytes) => { c.res.data(&bytes, &ct); }
                    Err(_) => { c.res.set_status(502); c.res.text("Bad Gateway"); }
                }
            }
            Err(e) => { 
                error!("Error proxying index request: {}", e);
                c.res.set_status(502); c.res.text("Bad Gateway"); 
            }
        }

        c
    }

    /// proxy to backend meta POST API
    pub async fn meta(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.c.health.add_search_count();
        let req = c.req.body_de_struct::<schema::search::MetaReq>().await;
        let result = match req {
            Ok(r) => c.c.ssr.search_page.search_api_meta(r).await,
            Err(_e) => {
                c.res.text("Bad Request: Failed to parse request body");
                c.res.set_status(400);
                return c;
            }
        };

        match result {
            Ok(resp) => {
                c.res.json_value(&serde_json::to_value(&resp).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                c.res.set_status(200);
            }
            Err(code) => { c.res.set_status(code); }
        }
        c
    }

    /// proxy to backend meta GET API
    pub async fn meta_get(mut c: Context<SiteContext>) -> Context<SiteContext> {
        c.c.health.add_search_count();
        let base = c.req.path.get_field("*").unwrap_or("".into());
        let query = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        let url = if query.is_empty() {
            base
        } else {
            format!("{}?{}", base, query)
        };
        println!("meta_get url: {}", url);
        let query = schema::search::MetaReq {
            url: url.to_string(),
        };
        let result = c.c.ssr.search_page.search_api_meta(query).await;

        match result {
            Ok(resp) => {
                c.res.json_value(&serde_json::to_value(&resp).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                c.res.set_status(200);
            }
            Err(code) => { c.res.set_status(code); }
        }
        c
    }
}