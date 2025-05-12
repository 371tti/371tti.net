use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionStatusAPI {
    pub accounts: Vec<AccountElement>,
    pub session_id: String,
    pub last_access_time: u64,
    pub now_user: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountElement {
    pub user_id: String,
    pub status: AccountStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AccountStatus {
    Active,
    SessionOut,
    ChangedPassword,
} 