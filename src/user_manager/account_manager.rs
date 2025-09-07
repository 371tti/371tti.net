use argon2::password_hash;
use dashmap::DashMap;
use dashmap::mapref::one::{Ref, RefMut};
use std::time::SystemTime;

use crate::user_manager::auth_manager::{Account, AccountData, AccountID, Accounts, SessionKey};

use crate::user_manager::auth_manager::ACCOUNT_DATA_VERSION;

impl Accounts {
    pub fn new() -> Self {
        Self {
            pool: DashMap::new(),
        }
    }

    /// get data from account id (returns a Ref)
    pub fn get(&self, id: &AccountID) -> Option<Ref<'_, AccountID, AccountData>> {
        self.pool.get(id)
    }

    /// get mutable data (DashMap uses RefMut)
    pub fn get_mut(&self, id: &AccountID) -> Option<RefMut<'_, AccountID, AccountData>> {
        self.pool.get_mut(id)
    }

    pub fn add_account(&self, account: &Account, password_hash: &[u8; 32], password_salt: &[u8; 16]) {
        let account_data = AccountData::new(account, password_hash, password_salt);
        self.pool.insert(account.id().clone(), account_data);
    }

    pub fn remove_account(&self, account: &AccountID) {
        self.pool.remove(account);
    }
}

impl AccountData {
    pub fn new(account: &Account, password_hash: &[u8; 32], password_salt: &[u8; 16]) -> Self {
        Self {
            account: account.clone(),
            password_hash: *password_hash,
            password_salt: *password_salt,
            session_ids: Vec::new(),
            created_at: SystemTime::now(),
            version: ACCOUNT_DATA_VERSION,
        }
    }

    /// verify password by its sha256 hash
    pub fn verify_password_hash(&self, password_hash: &[u8; 32]) -> bool {
        self.password_hash == *password_hash
    }

    /// add session to this account
    pub fn add_session(&mut self, session: SessionKey) {
        self.session_ids.push(session);
    }

    /// remove session from this account
    pub fn remove_session(&mut self, session: &SessionKey) {
        self.session_ids.retain(|s| s != session);
    }

    /// list all sessions of this account
    pub fn list_sessions(&self) -> &Vec<SessionKey> {
        &self.session_ids
    }
}