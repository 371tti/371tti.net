use dashmap::{mapref::one::RefMut, DashMap};
use argon2::Argon2;
use std::time::{Duration, SystemTime};


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
    pub session_timeout: Duration,
    pub password_pepper: String,
    pub password_hasher: Argon2<'static>,
}

use crate::user_manager::hash_config::HashConfig;

/// length of password hash
/// 32 bytes (256 bits)
pub const HASH_LEN: usize = 32;

///  account data version
/// for future
pub const ACCOUNT_DATA_VERSION: u32 = 1;

impl AuthManager {
    /// new instance of AuthManager
    /// hash_salt: 16bytes random
    /// hash_stretching: number of iterations for password hashing
    pub fn new( session_timeout: Duration, hash_config: HashConfig) -> Self {
        Self {
            sessions: Sessions::new(),
            accounts: Accounts::new(),
            session_timeout,
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
            )
        }
    }
    /// verify account
    pub fn verify_account(&self, id: &AccountID, password: &str) -> VerifyResult {
        self.accounts.get(id).map(|account_data| {
            let password_hash = self.hash_password(password, &account_data.password_salt);
            if account_data.verify_password_hash(&password_hash) {
                VerifyResult::Success
            } else {
                VerifyResult::InvalidPassword
            }
        }).unwrap_or(VerifyResult::NoSuchAccount)
    }

    /// check session
    pub fn check_session(&self, session: &SessionKey) -> Option<RefMut<'_, SessionKey, SessionsData>> {
        self.sessions.get_mut(session)
    }

    /// add_new_account
    pub fn add_new_account(&self, account: &Account, password_hash: &[u8; HASH_LEN], password_salt: &[u8; 16]) {
        self.accounts.add_account(account, password_hash, password_salt);
    }

    
    /// create new session
    pub fn create_session(&self) -> SessionKey {
        let key = self.sessions.add_session();
        key
    }

    fn hash_password(&self, password: &str, salt: &[u8; 16]) -> [u8; HASH_LEN] {
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
    pub pool: DashMap<AccountID, AccountData>,
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


/// account id
/// only ascii
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct AccountID(pub String);

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Account {
    Admin(AccountID),
    Normal(AccountID),
}

impl Account {
    pub fn id(&self) -> &AccountID {
        match self {
            Account::Admin(id) => id,
            Account::Normal(id) => id,
        }
    }
    
}

pub struct SessionsData {
    /// 複数アカウントログイン対応
    pub accounts: Vec<AccountSession>,
    /// 現在操作しているアカウントのインデックス
    pub now_account_index: Option<usize>,
    pub created_at: SystemTime,
    pub last_accessed_at: SystemTime,
}

pub struct AccountSession {
    pub account: Account,
    pub status: AccountSessionStatus,
}

pub enum AccountSessionStatus {
    /// 時間によっては有効なアカウント
    Enable(SystemTime), // 最終ログイン時間
    /// 無効なアカウント
    Logout,
}

pub struct AccountData {
    pub account: Account,
    /// sha256 hash
    pub password_hash: [u8; 32],
    pub password_salt: [u8; 16],
    pub session_ids: Vec<SessionKey>,
    pub created_at: SystemTime,
    pub version: u32, // for future
}

