use std::time::SystemTime;

use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};

use crate::user_manager::auth_manager::{SessionKey, Sessions, SessionsData};
use rand::RngCore;

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
        let session = SessionsData::new();
        self.pool.insert(key.clone(), session);
        key
    }
}

impl SessionKey {
    pub fn new() -> Self {
        let mut rng = rand::rngs::OsRng;
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        SessionKey(key)
    }
}

impl SessionsData {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            now_account_index: None,
            created_at: SystemTime::now(),
            last_accessed_at: SystemTime::now(),
        }
    }
}