use std::collections::HashMap;

use chrono::Utc;

use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use base64::{Engine, engine::general_purpose};

use crate::user_manager::auth::{AccountSession, SessionKey, Sessions, SessionsData};

impl Sessions {
    pub fn new() -> Self {
        Self {
            pool: DashMap::new(),
        }
    }

    pub fn get(&self, key: &SessionKey) -> Option<Ref<'_, SessionKey, SessionsData>> {
        self.pool.get(key)
    }

    pub fn get_mut(&self, key: &SessionKey) -> Option<RefMut<'_, SessionKey, SessionsData>> {
        self.pool.get_mut(key)
    }

    pub fn add_session(&self) -> SessionKey {
        let key = SessionKey::new();
        if self.pool.contains_key(&key) {
            // extremely rare case
            return self.add_session();
        }
        let session = SessionsData::new();
        self.pool.insert(key.clone(), session);
        key
    }

    pub fn remove_session(&self, key: &SessionKey,) {
        self.pool.remove(key);
    }
}

impl SessionKey {
    pub fn dummy() -> Self {
        SessionKey([0u8; 32])
    }

    pub fn new() -> Self {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).expect("generate random session key");
        SessionKey(key)
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match general_purpose::STANDARD.decode(s) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Some(SessionKey(key))
            },
            _ => None,
        }
    }

    pub fn as_base64(&self) -> String {
        general_purpose::STANDARD.encode(&self.0)
    }

    /// 外部に公開してもよい 先頭数文字を省略した形
    pub fn to_safe_string(&self) -> String {
        let b64 = self.as_base64();
        if b64.len() <= 8 {
            b64
        } else {
            format!("...{}", &b64[8..])
        }
    }
}

impl SessionsData {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            now_account_index: None,
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
            data_storage: HashMap::new(),
        }
    }

    pub fn push_account(&mut self, account_session: AccountSession) -> usize {
        self.accounts.push(account_session);
        self.accounts.len() - 1
    }
}