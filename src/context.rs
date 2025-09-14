use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::Duration as ChronoDuration;


use kurosabi::{context::ContextMiddleware, kurosabi::Context};
use log::info;
use mongodb::options::ClientOptions;
use mongodb::Client;

use crate::{config::MainConfig, page_generator::PageGenerator, user_manager::auth_manager::{AccountID, AuthManager, SessionKey}};


/// Cookie key for session management
pub const SESSION_COOKIE_KEY: &str = "session_key";
/// Cookie max age in seconds
/// 1 year
pub const SESSION_COOKIE_MAX_AGE: i64 = 60 * 60 * 24 * 365;

pub const ACCOUNT_COLLECTION_NAME: &str = "accounts";

#[derive(Clone)]
pub struct SiteContext {
    /// Server-Side Rendering engine
    pub ssr: Arc<PageGenerator>,
    /// Authentication manager
    pub auth: Arc<AuthManager>,
    /// Main configuration
    pub main_config: Arc<Mutex<MainConfig>>,
    // db
    pub db_client: Arc<mongodb::Database>,
    /// Current session key, if any
    pub session_key: Option<SessionKey>,
    pub user_id: Option<AccountID>,
}

impl SiteContext {
    pub async fn new(config: PathBuf) -> Self {
        info!("Loading config from {:?}", config);
        let config = MainConfig::load_from_file(&config);
        info!("Config loaded");
        let ssr = Arc::new(PageGenerator::new()); // Assuming PageGenerator has a new() method

        let client_options = ClientOptions::parse(&config.database_url).await.expect("Failed to parse MongoDB connection string");
        let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");
        let db = client.database(&config.db_name);
        let db_client = Arc::new(db);

        let auth = Arc::new(AuthManager::new(
            ChronoDuration::seconds(config.session_timeout as i64),
            ChronoDuration::seconds(config.account_timeout as i64),
            config.hash_config.clone(),
            db_client.clone(),
        ).await);
        
        Self { 
            ssr, 
            auth, 
            main_config: Arc::new(Mutex::new(config)),
            session_key: None,
            user_id: None,
            db_client,
        }
    }
}

#[async_trait::async_trait]
impl ContextMiddleware<SiteContext> for SiteContext {
    /// セッション管理用
    async fn before_handle(mut ctx: Context<SiteContext>) -> Context<SiteContext> {
        let mut needs_new_session = true;

        if let Some(cookie) = ctx.req.header.get_cookie(SESSION_COOKIE_KEY) {
            if let Some(key) = SessionKey::from_str(cookie) {
                if let Some(session) = ctx.c.auth.check_session(&key).await {
                    // set context session key
                    ctx.c.user_id = session
                        .now_account_index
                        .and_then(|idx| { 
                            session.accounts.get(idx)
                                .map(|acc_sess| acc_sess.get_account_id().clone()) 
                            });
                    ctx.c.session_key = Some(key.clone());
                    needs_new_session = false;
                }
            }
        }

        if needs_new_session {
            let session_key = ctx.c.auth.create_session();
            // set context session key
            ctx.c.session_key = Some(session_key.clone());
            ctx.res
                .header
                .set_cookie_with_params(
                    SESSION_COOKIE_KEY, 
                    &session_key.as_base64(),
                    true,
                    true,
                    Some("/"),
                    Some("Lax"),
                    Some(SESSION_COOKIE_MAX_AGE),
                    None,
                    None,
                    None,
                );
        }

        ctx
    }
}

