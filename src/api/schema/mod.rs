use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::user_manager::auth_manager::{AccountID, AccountSession};

#[derive(Clone, Serialize)]
pub struct SessionState {
    /// Whether the user is logged in
    pub is_logged_in: bool,
    /// The currently logged-in account, if any
    pub logged_account: Option<AccountID>,
    /// All authenticated accounts (for multi-login scenarios)
    pub authenticated_accounts: Vec<AccountSession>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct LoginReq {
    /// Account ID
    pub account_id: AccountID,
    /// If the requested account is in the session's valid accounts list, the password can be skipped.
    /// just like account switching from user view
    pub skip_password: bool,
    /// Password for the account
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginRes {
    pub success: bool,
    pub message: String,
    pub redirect_to: Option<String>,
}

// --- Search API schema ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub url: String,
    pub score: f64,
    pub length: u64,
    pub description: Option<String>,
    pub title: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum SearchApiResult {
    #[serde(rename = "true")]
    Success {
        query: String,
        tokenize_query: Vec<String>,
        algorithm: String,
        range: Range<usize>,
        results: Vec<ResEntry>,
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResEntry {
    pub url: Box<str>,
    pub title: Box<str>,
    pub favicon: Option<Box<str>>,
    pub tags: Vec<Box<str>>,
    pub descriptions: Box<str>,
    pub score: f64,
    pub point: f64,
    pub length: u64,
    pub id: usize,
    pub index_id: usize,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexReq {
    pub url: String,
    pub title: Option<String>,
    pub favicon: Option<String>,
    /// タグは空でも良い
    /// 例: ["wiki", "blog"]
    /// 使用可能なタグ:
    /// - "wiki": ウィキペディアなどの百科事典
    /// - "news": ニュースサイト
    /// - "sns": ソーシャルメディア
    /// - "blog": ブログ
    /// - "forum": フォーラム
    /// - "shopping": ショッピングサイト
    /// - "academic": 学術論文
    /// - "tools": ツール系サイト
    pub tags: Vec<String>,
    pub descriptions: Option<String>,
}