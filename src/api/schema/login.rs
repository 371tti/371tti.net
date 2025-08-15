pub struct LoginReqAPI {
    pub username: String,
    /// セッションが有効期限内の場合、Noneでログインできる。
    pub password: Option<String>,
}

pub struct LoginResAPI {
    pub success: bool,
    pub now_user: Option<String>,
}