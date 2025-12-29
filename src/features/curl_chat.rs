use tokio::sync::broadcast;

use crate::user_manager::auth::SessionKey;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

pub struct CurlChatState {
    pub broadcast_tx: broadcast::Sender<CurlChatPacket>,
    pub broadcast_rx: broadcast::Receiver<CurlChatPacket>,
}

#[derive(Clone)]
pub struct CurlChatPacket {
    pub username: String,
    pub message: String,
    pub session_key: SessionKey,
}

impl CurlChatState {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(100);
        let tx_clone = tx.clone();
        tokio::spawn(async move {

            let mut interval = tokio::time::interval(Duration::from_secs(60 * 1));

            // 起動直後に送らず、最初の5分経過を待つ
            interval.tick().await;

            loop {
            interval.tick().await;

            let now: DateTime<Utc> = SystemTime::now().into();
            let _ = tx_clone.send(CurlChatPacket {
                username: "system".to_string(),
                message: format!("time: {}", now.format("%Y-%m-%d %H:%M:%S UTC")),
                session_key: SessionKey::dummy(),
            });
            }
        });
        CurlChatState {
            broadcast_tx: tx,
            broadcast_rx: rx,
        }
    }
}