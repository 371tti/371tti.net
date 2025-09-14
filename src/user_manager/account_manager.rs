use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::one::{MappedRef, MappedRefMut, Ref, RefMut};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use mongodb::bson::{self, doc};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use futures::TryStreamExt;

use crate::context::ACCOUNT_COLLECTION_NAME;
use crate::user_manager::auth_manager::{AccountData, AccountID, AccountSession, AccountSessionStatus, Accounts, SessionKey};

use crate::user_manager::auth_manager::ACCOUNT_DATA_VERSION;

// DB系の処理ここに
impl Accounts {
    pub async fn new(db_client: Arc<mongodb::Database>) -> Self {
        #[derive(Serialize, Deserialize)]
        struct AccountDataID {
            pub account_id: AccountID,
        }

        let col_full: Collection<AccountData> = db_client.collection(ACCOUNT_COLLECTION_NAME);
        let col = col_full.clone_with_type::<AccountDataID>();

        let mut cursor = col.find(doc! {  }).await.expect("Failed to fetch account IDs");
        let pool = DashMap::new();

        while let Some(doc) = cursor.try_next().await.expect("Failed to get next document") {
            pool.insert(doc.account_id, None);
        }

        info!{ "Loaded {} account IDs from database", pool.len() }


        Self {
            pool,
            db_client,
        }
    }

    /// get data from account id (returns a Ref)
    pub async fn get(&self, id: &AccountID) -> Option<MappedRef<'_, AccountID, std::option::Option<AccountData>, AccountData>> {
        let r = self.pool.get(id)?;
        if r.is_none() {
            // drop しないとdeadlockする
            drop(r);
            self.load_account(id).await;
            let r = self.pool.get(id)?;
            if r.is_none() {
                return None;
            }
            return Some(Ref::map(r, |opt| opt.as_ref().unwrap()));
        }
        Some(Ref::map(r, |opt| opt.as_ref().expect("AccountData should be loaded")) )
    }

    /// get mutable data (DashMap uses RefMut)
    pub async fn get_mut(&self, id: &AccountID) -> Option<MappedRefMut<'_, AccountID, std::option::Option<AccountData>, AccountData>> {
        let r = self.pool.get_mut(id)?;
        if r.is_none() {
            // drop しないとdeadlockする
            drop(r);
            self.load_account(id).await;
            let r = self.pool.get_mut(id)?;
            if r.is_none() {
                return None;
            }
            return Some(RefMut::map(r, |opt| opt.as_mut().unwrap()));
        }
        Some(RefMut::map(r, |opt| opt.as_mut().unwrap()))
    }

    pub async fn load_account(&self, id: &AccountID) -> Option<()> {
        debug!("Loading account data for ID: {}", id.as_str());
        let col_full: Collection<AccountData> = self.db_client.collection(ACCOUNT_COLLECTION_NAME);
        let account_data = col_full.find_one(doc! {"account_id": &id.0}).await.map_err(|e| {
            error!("Database error while fetching account {}: {}", id.as_str(), e);
        }).ok()??;
        self.pool.insert(id.clone(), Some(account_data));
        debug!("Account data for ID: {} loaded successfully", id.as_str());
        Some(())
    }

    pub async fn save_account(&self, id: &AccountID) -> Option<()> {
        let account_data = self.get(id).await?.clone();
        if account_data.is_saved {
            return Some(());
        }
        let col_full: Collection<AccountData> = self.db_client.collection(ACCOUNT_COLLECTION_NAME);
        let filter = doc! { "account_id": &id.0 };
        let update = doc! { "$set": bson::to_bson(&account_data).ok()? };
        let res = col_full.update_one(filter, update).await.map_err(|e| {
            error!("Database error while saving account {}: {}", id.as_str(), e);
        }).ok()?;
        if res.matched_count == 0 {
            // insert if not exists
            col_full.insert_one(&account_data).await.map_err(|e| {
                error!("Database error while inserting account {}: {}", id.as_str(), e);
            }).ok()?;
        }
        if let Some(mut acc) = self.get_mut(id).await {
            acc.is_saved = true;
        }
        Some(())
    }

    pub fn contains_account(&self, id: &AccountID) -> bool {
        self.pool.contains_key(id)
    }

    pub async fn insert_account(&self, account: AccountData) {
        let account_id = account.account_id.clone();
        self.pool.insert(account_id.clone(), Some(account));
        self.save_account(&account_id).await;
    }

    pub async fn link_account_session(&self, account_id: &AccountID, session: &SessionKey) -> bool {
        if let Some(mut account_data) = self.get_mut(account_id).await {
            account_data.add_session(session.clone());
            account_data.is_saved = false;
            true
        } else {
            false
        }
    }

    pub async fn update_last_accessed(&self, account_id: &AccountID) -> bool {
        if let Some(mut account_data) = self.get_mut(account_id).await {
            account_data.last_accessed_at = Utc::now();
            account_data.is_saved = false;
            true
        } else {
            false
        }
    }

    pub fn remove_account(&self, account: &AccountID) {
        self.pool.remove(account);
    }
}

impl AccountID {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AccountData {
    pub fn new(account: &AccountID, password_hash: &[u8; 32], password_salt: &[u8; 16]) -> Self {
        Self {
            account_id: account.clone(),
            password_hash: *password_hash,
            password_salt: *password_salt,
            session_ids: Vec::new(),
            created_at: Utc::now(),
            version: ACCOUNT_DATA_VERSION,
            is_saved: false,
            last_accessed_at: Utc::now(),
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

impl AccountSession {
    pub fn new(account: &AccountID) -> Self {
        Self {
            account_id: account.clone(),
            status: AccountSessionStatus::Logout,
        }
    }

    pub fn get_time(&self) -> Option<DateTime<Utc>> {
        match &self.status {
        AccountSessionStatus::Enable(t) => Some(*t),
            AccountSessionStatus::Logout => None,
        }
    }

    pub fn set_time(&mut self, t: DateTime<Utc>) {
        self.status = AccountSessionStatus::Enable(t);
    }

    pub fn get_account_id(&self) -> &AccountID {
        &self.account_id
    }
}