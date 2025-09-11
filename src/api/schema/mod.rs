use chrono::{DateTime, Utc};
use serde::Serialize;

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

pub struct LoginReq {
    /// Account ID
    pub account_id: String,
    /// If the requested account is in the session's valid accounts list, the password can be skipped.
    /// just like account switching from user view
    pub skip_password: bool,
    /// Password for the account
    pub password: String,
}

pub struct LoginRes {
    pub success: bool,
    pub message: String,
    pub redirect_to: Option<String>,
}