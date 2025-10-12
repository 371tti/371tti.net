use kurosabi::kurosabi::Context;

use crate::{api::schema::auth::LoginReq, context::SiteContext};

pub struct AuthAPI;

impl AuthAPI {
    pub async fn session(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if let Some(session_state) = c.c.req_session_state() {
            let serded_state = serde_json::to_value(&session_state).unwrap_or(serde_json::json!({"error": "Failed to serialize session state"}));
            c.res.json_value(&serded_state);
        } else {
            c.res.set_status(401);
            c.res.json_value(&serde_json::json!({"error": "No valid session"}));
        }
        c
    }

    pub async fn auth(mut c: Context<SiteContext>) -> Context<SiteContext> {
        match c.req.body_de_struct::<LoginReq>().await {
            Ok(login_req) => {
                match c.c.req_login(login_req).await {
                    Ok(res) => {
                        c.res.json_value(&serde_json::to_value(&res).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                    },
                    Err((res, status)) => {
                        c.res.set_status(status);
                        c.res.json_value(&serde_json::to_value(&res).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                    },
                }
                c
            },
            Err(e) => {
                c.res.set_status(400);
                c.res.json_value(&serde_json::json!({"error": format!("Failed to parse request body: {}", e)}));
                return c;
            },
        }
    }
}