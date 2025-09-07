use std::{path::PathBuf, sync::{Arc, Mutex}, time::{Duration, SystemTime}};


use kurosabi::{context::ContextMiddleware, kurosabi::Context};
use log::info;

use crate::{config::MainConfig, page_generator::PageGenerator, user_manager::auth_manager::{AuthManager, SessionKey}};

#[derive(Clone)]
pub struct SiteContext {
    /// Server-Side Rendering engine
    pub ssr: Arc<PageGenerator>,
    /// Authentication manager
    pub auth: Arc<AuthManager>,
    /// Main configuration
    pub main_config: Arc<Mutex<MainConfig>>,
    /// Current session key, if any
    pub session_key: Option<SessionKey>,
}

impl SiteContext {
    pub fn new(config: PathBuf) -> Self {
        info!("Loading config from {:?}", config);
        let config = MainConfig::load_from_file(&config);
        info!("Config loaded");
        let ssr = Arc::new(PageGenerator::new()); // Assuming PageGenerator has a new() method
        let auth = Arc::new(AuthManager::new(
            Duration::from_secs(config.session_timeout),
            config.hash_config.clone(),
        ));
        Self { 
            ssr, 
            auth, 
            main_config: Arc::new(Mutex::new(config)),
            session_key: None,
        }
    }
}

/// Cookie key for session management
pub const SESSION_COOKIE_KEY: &str = "session_key";
/// Cookie max age in seconds
/// 1 year
pub const SESSION_COOKIE_MAX_AGE: i64 = 60 * 60 * 24 * 365;

#[async_trait::async_trait]
impl ContextMiddleware<Context<SiteContext>> for SiteContext {
    /// セッション管理用
    async fn before_handle(mut ctx: Context<SiteContext>) -> Context<SiteContext> {
        let mut needs_new_session = true;

        if let Some(cookie) = ctx.req.header.get_cookie(SESSION_COOKIE_KEY) {
            if let Some(key) = SessionKey::from_str(cookie) {
                if let Some(mut session) = ctx.c.auth.check_session(&key) {
                    // set context session key
                    ctx.c.session_key = Some(key.clone());
                    session.last_accessed_at = SystemTime::now();
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

