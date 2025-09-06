use dashmap::DashMap;
use sha2::{Digest, Sha256};
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
    pub hash_salt: [u8; 16],
    pub hash_stretching: u8,
    pub session_timeout: Duration,
}

impl AuthManager {
    /// new instance of AuthManager
    /// hash_salt: 16bytes random
    /// hash_stretching: number of iterations for password hashing
    pub fn new( session_timeout: Duration, hash_salt: [u8; 16], hash_stretching: u8 ) -> Self {
        Self {
            sessions: Sessions::new(),
            accounts: Accounts::new(),
            hash_salt,
            hash_stretching,
            session_timeout,
        }
    }
    /// verify account
    fn verify_account(&self, id: &AccountID, password: &str) -> VerifyResult {
        self.accounts.get(id).map(|account_data| {
            let password_hash = self.password_hasher(password);
            if account_data.verify_password_hash(&password_hash) {
                VerifyResult::Success
            } else {
                VerifyResult::InvalidPassword
            }
        }).unwrap_or(VerifyResult::NoSuchAccount)
    }

    /// check session
    fn check_session(&self, session: &SessionKey) -> bool {
       todo!()
    }

    /// add_new_account
    fn add_new_account(&self, account: &Account, password_hash: &[u8; 32]) {
        self.accounts.add_account(account, password_hash);
    }

    
    /// create new session
    fn create_session(&self) -> SessionKey {
        let key = self.sessions.add_session();
        key
    }

    fn password_hasher(&self, password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.hash_salt);
        hasher.update(password);
        for _i in 0..self.hash_stretching {
            let hash_result = hasher.finalize_reset();
            hasher.update(&hash_result);
        }
        hasher.finalize().into()
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
    pub session_ids: Vec<SessionKey>,
    pub created_at: SystemTime,
}

