use std::{collections::HashMap, sync::Arc};

use dashmap::{mapref::one::RefMut, DashMap};
use argon2::Argon2;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use serde::{Deserialize, Serialize};


/// 認証マネージャーの実装
/// Cookieからセッションとアカウントの管理をするやつ
/// 必要な機能
/// - ログイン
/// - セッションライフ管理
/// - アカウント管理
/// - save/load
/// 
/// 複数ログインのためセッションが複数のアカウントの参照をもってるかんじの構造で
/// セッションからのアカウント操作と
/// アカウントからのセッション操作とがいる
pub struct AuthManager {
    pub sessions: Sessions,
    pub accounts: Accounts,
    pub session_timeout: ChronoDuration,
    pub account_timeout: ChronoDuration,
    pub password_pepper: String,
    pub password_hasher: Argon2<'static>,
}

use crate::user_manager::hash_config::HashConfig;
use serde::{Serializer, Deserializer};
use base64::{engine::general_purpose, Engine};
use serde::de::Error as DeError;

/// length of password hash
/// 32 bytes (256 bits)
pub const HASH_LEN: usize = 32;

///  account data version
/// for future
pub const ACCOUNT_DATA_VERSION: u32 = 1;

/// If the number of sessions exceeds this threshold, a garbage collection will be triggered.
pub const SESSION_GC_THRESHOLD: usize = 128;

impl AuthManager {
    /// new instance of AuthManager
    /// hash_salt: 16bytes random
    /// hash_stretching: number of iterations for password hashing
    pub async fn new( session_timeout: ChronoDuration, account_timeout: ChronoDuration, hash_config: HashConfig, db_client: Arc<mongodb::Database>) -> Self {
        Self {
            sessions: Sessions::new(),
            accounts: Accounts::new(
                db_client
            ).await,
            session_timeout,
            account_timeout,
            password_pepper: hash_config.pepper,
            password_hasher: Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2::Params::new(
                    hash_config.memory_kib,
                    hash_config.time_cost,
                    hash_config.lanes,
                    Some(HASH_LEN)
                ).expect("argon2 hash params")
            ),
        }
    }
    /// verify account
    pub async fn verify_account(&self, id: &AccountID, password: &str) -> VerifyResult {
        self.accounts.get(id).await.map(|account_data| {
            let password_hash = self.hash_password(password, &account_data.password_salt);
            if account_data.verify_password_hash(&password_hash) {
                VerifyResult::Success
            } else {
                VerifyResult::InvalidPassword
            }
        }).unwrap_or(VerifyResult::NoSuchAccount)
    }

    /// check session
    /// セッションの有効性を確認します。無効なら削除してNoneを返す。
    /// これが実行された直後のセッションデータは有効性が保証される。
    pub async fn check_session(&self, session_key: &SessionKey) -> Option<RefMut<'_, SessionKey, SessionsData>> {
        let session = self.sessions.get_mut(session_key);
        if let Some(mut s) = session {
            let now: DateTime<Utc> = Utc::now();
            if now.signed_duration_since(s.last_accessed_at) <= self.session_timeout {
                // アクセス時間更新
                s.last_accessed_at = now;
                s.now_account_index.map(|idx| {
                    if let Some(account_session) = s.accounts.get_mut(idx) {
                        if let Some(time) = account_session.get_time() {
                            if now.signed_duration_since(time) <= self.account_timeout {
                                // アクセス時間更新
                                account_session.set_time(now);
                            } else {
                                // timeout
                                s.now_account_index = None;
                            }
                        } else {
                            // account is logout
                            s.now_account_index = None;
                        }
                    } else {
                        // index out of range
                        s.now_account_index = None;
                    }
                });
                Some(s)
            } else {
                // del session
                // unlink session from accounts
                for acc_session in s.accounts.iter(){
                    if let Some(mut account_data) = self.accounts.get_mut(&acc_session.account_id).await {
                        account_data.remove_session(&session_key);
                    }
                }
                // session expired
                None
            }
        } else {
            None
        }
    }

    pub async fn session_gc_task(&self) {
        let keys_to_remove: Vec<SessionKey> = self.sessions.pool.iter()
            .filter_map(|entry| {
                let now = Utc::now();
                if now.signed_duration_since(entry.value().last_accessed_at) > self.session_timeout {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        if keys_to_remove.len() < SESSION_GC_THRESHOLD {
            log::info!("Session GC: no need to run GC, only {} timeout sessions", keys_to_remove.len());
            return;
        }

        log::info!("Session GC: removing {} timeout sessions", keys_to_remove.len());
        for key in keys_to_remove {
            if let Some(session) = self.sessions.get_mut(&key) {
                // unlink session from accounts
                for acc_session in session.accounts.iter(){
                    if let Some(mut account_data) = self.accounts.get_mut(&acc_session.account_id).await {
                        account_data.remove_session(&key);
                    }
                }
            }
            self.sessions.remove_session(&key);
        }
    }
    
    /// create new session
    pub fn create_session(&self) -> SessionKey {
        let key = self.sessions.add_session();
        key
    }

    pub async fn add_account(&self, id: &AccountID, password: &str) -> Option<[u8; 16]> {
        if self.accounts.contains_account(id) {
            // already exists
            return None;
        }
        // generate random salt
        let mut salt = [0u8; 16];
        getrandom::fill(&mut salt).expect("generate random salt");
        let password_hash = self.hash_password(password, &salt);
        let account_data = AccountData::new(id, &password_hash, &salt);
        self.accounts.insert_account(account_data).await;
        Some(salt)
    }

    pub fn hash_password(&self, password: &str, salt: &[u8; 16]) -> [u8; HASH_LEN] {
        // Use Argon2 to derive a fixed-length raw hash (32 bytes)
        let mut out = [0u8; HASH_LEN];
        // combine salt and pepper
        // salt: 16bytes
        let mut adv = Vec::new();
        adv.extend_from_slice(salt);
        adv.extend_from_slice(self.password_pepper.as_bytes());

        self.password_hasher.hash_password_into(password.as_bytes(), &adv, &mut out).expect("argon2 hash");
        out
    }
}

pub enum VerifyResult {
    Success,
    NoSuchAccount,
    InvalidPassword,
}

/// Account Manager
pub struct Accounts {
    pub pool: DashMap<AccountID, Option<AccountData>>,
    pub db_client: Arc<mongodb::Database>,
}

/// Session Manager
pub struct Sessions {
    pub pool: DashMap<SessionKey, SessionsData>,
}

/// session key
/// random 32bytes
/// so it is 256bit
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct SessionKey(pub [u8; 32]);

impl Serialize for SessionKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = general_purpose::STANDARD.encode(&self.0);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for SessionKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid SessionKey length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(SessionKey(arr))
    }
}


/// account id
/// only ascii
#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct AccountID(pub String);

pub struct SessionsData {
    /// 複数アカウントログイン対応
    pub accounts: Vec<AccountSession>,
    /// 現在操作しているアカウントのインデックス
    pub now_account_index: Option<usize>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    /// その他のデータ保存用領域
    pub data_storage: HashMap<String, String>,
}

#[derive(Clone, Serialize)]
pub struct AccountSession {
    pub account_id: AccountID,
    pub status: AccountSessionStatus,
}

#[derive(Clone, Serialize)]
pub enum AccountSessionStatus {
    /// 時間によっては有効なアカウント
    Enable(DateTime<Utc>), // 最終ログイン時間
    /// 無効なアカウント
    Logout,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct AccountData {
    pub account_id: AccountID,
    #[serde(
        serialize_with = "as_base64",
        deserialize_with = "from_base64"
    )]
    pub password_hash: [u8; 32],
    #[serde(
        serialize_with = "as_base64",
        deserialize_with = "from_base64"
    )]
    pub password_salt: [u8; 16],
    pub session_ids: Vec<SessionKey>,
    pub created_at: DateTime<Utc>,
    pub version: u32, // for future
    #[serde(skip)]
    #[serde(default = "default_true")]
    pub is_saved: bool,
    pub last_accessed_at: DateTime<Utc>,
}

fn default_true() -> bool {
    true
}

// base64 helpers for serde

fn as_base64<S, const N: usize>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&encoded)
}

fn from_base64<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let decoded = general_purpose::STANDARD
        .decode(&s)
        .map_err(DeError::custom)?;
    if decoded.len() != N {
        return Err(DeError::custom(format!("Invalid length: expected {}, got {}", N, decoded.len())));
    }
    let mut arr = [0u8; N];
    arr.copy_from_slice(&decoded);
    Ok(arr)
}
