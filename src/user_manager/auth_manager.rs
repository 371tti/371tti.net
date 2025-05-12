use dashmap::DashMap;
use kurosabi::{request::Req, response::Res};
use rand::rngs::OsRng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};

use crate::api::schema::select_login::{AccountElement, AccountStatus, SessionStatusAPI};

pub struct AuthManager {
    pub data: DashMap<String, AuthInfo>,
    pub sessions: DashMap<String, Session>, // session_id -> user_id
}

impl AuthManager {
    const SESSION_ID_LENGTH: usize = 64; // セッションIDの長さ
    const USER_SESSION_TIMEOUT: u64 = 60 * 60 * 24; // ユーザーセッションのタイムアウト時間（秒）
    const SESSION_TIMEOUT: u64 = 60 * 60 * 24; // セッションのタイムアウト時間（秒）

    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
            sessions: DashMap::new(),
        }
    }

    pub fn add_user(&self, user_name: String, password: String) {
        let auth_info = AuthInfo::new(password.clone());
        self.data.insert(user_name.clone(), auth_info);
    }

    pub fn all_session_logout(&self, user_name: &str) {
        if let Some(user) = self.data.get(user_name) {
            for session_id in user.sessions.iter() {
                if let Some(mut session) = self.sessions.get_mut(session_id) {
                    session.now_user = None; // セッションのユーザーを削除
                self.sessions.get_mut(session_id).unwrap().del_user(user_name); // セッションからユーザーを削除
                }
            }
        }
    }

    pub fn api_session_status(&self, session_id: &str) -> Option<SessionStatusAPI> {
        if let Some(session) = self.sessions.get(session_id) {
            let mut accounts = Vec::new();
            for user in session.users.iter() {
                let status = if user.last_access_time < chrono::Utc::now().timestamp() as u64 - Self::USER_SESSION_TIMEOUT {
                    AccountStatus::SessionOut
                } else if let Some(auth_info) = self.data.get(&user.user_id) {
                    if user.cache_password != auth_info.password {
                        AccountStatus::ChangedPassword
                    } else {
                        AccountStatus::Active
                    }
                } else {
                    AccountStatus::ChangedPassword // or another appropriate status if user not found
                };
                accounts.push(AccountElement { user_id: user.user_id.clone(), status });
            }
            Some(SessionStatusAPI {
                accounts,
                session_id: session_id.to_string(),
                last_access_time: session.last_access_time,
                now_user: session.now_user.clone(),
            })
        } else {
            None
        }
    }

    pub fn drop_useless_session(&self) {
        let now_time = chrono::Utc::now().timestamp() as u64;
        for entry in self.sessions.iter_mut() {
            let (session_id, session) = entry.pair();
            if session.last_access_time < now_time - Self::SESSION_TIMEOUT {
                for user in session.users.iter() {
                    if let Some(mut user) = self.data.get_mut(&user.user_id) {
                        user.sessions.retain(|s| s != session_id); // セッションIDを削除
                    }
                }
                // セッションの有効期限が切れた場合は削除
                self.sessions.remove(session_id);
            }
        }
    }

    pub fn check_session(&self, req: &Req, res: &mut Res) -> (Option<String>, String){
        let now_time = chrono::Utc::now().timestamp() as u64;
        let mut session_id: String = req.header.get_cookie("session_id").unwrap_or("").to_string();
        // 有効なセッションを確保
        if !self.sessions.contains_key(&session_id) {
            let gen_id = self.generate_session_id();
            self.sessions.insert(gen_id.clone(), Session::new());
            session_id = gen_id;
        }
        let mut session = self.sessions.get_mut(&session_id).unwrap();
        session.last_access_time = now_time;
        res.header.set_cookie("session_id", &session_id);

        // ユーザー関連とのセッションの有効期限を確認
        let mut force_logout = false;
        if let Some(now_user) = session.now_user.clone() {
            if let Some(user) = session.get_mut_user(&now_user) {
                if user.last_access_time < now_time - Self::USER_SESSION_TIMEOUT {
                    force_logout = true; // セッションの有効期限が切れた場合はログアウト
                }
                if let Some(mut user_data) = self.data.get_mut(&now_user) {
                    if user_data.password != user.cache_password {
                        force_logout = true; // パスワードが変更された場合はログアウト
                    }
                    user_data.last_access_time = now_time; // アクセス時間を更新(data側)
                } else {
                    force_logout = true; // ユーザーが見つからない場合はログアウト
                }
                user.last_access_time = now_time; // アクセス時間を更新(session側)
            } else {
                force_logout = true; // ユーザーが見つからない場合はログアウト
            }
        }
        if force_logout {
            session.now_user = None;
        }
        return (session.now_user.clone(), session_id); // Return the now_user as an Option<String> and the session_id
    }

    pub fn auth(&self, res: Req, user_name: String, password: String) -> bool {
        if let Some(mut auth_info) = self.data.get_mut(&user_name) {
            if auth_info.password == password {
                let session_id = res.header.get_cookie("session_id");
                match session_id {
                    Some(id) => {
                        let mut session = self.sessions.get_mut(id).unwrap();
                        session.now_user = Some(user_name.clone());
                        let time = session.last_access_time;
                        auth_info.sessions.push(id.to_string());
                        session.users.push(SessionElement::new(
                            user_name.clone(),
                            time,
                            time,
                            password.clone(),
                        ));
                    }
                    None => {
                        return false; // session_idがない場合は認証失敗
                    }
                }
            }
        }
        false
    }

    pub fn generate_session_id(&self) -> String {
        let mut bytes = [0u8; Self::SESSION_ID_LENGTH];
        let mut rng = OsRng;
        rng.fill_bytes(&mut bytes);
        let id = general_purpose::URL_SAFE_NO_PAD.encode(&bytes);
        if self.sessions.contains_key(&id) {
            return self.generate_session_id();
        }
        id
    }
}



pub struct AuthInfo {
    pub password: String,
    pub cookie: String,
    pub last_access_time: u64,
    pub sessions: Vec<String>, // セッションIDのリスト
}

impl AuthInfo {
    pub fn new(password: String) -> Self {
        Self {
            password,
            cookie: String::new(),
            last_access_time: 0,
            sessions: Vec::new(),
        }
    }
}

pub struct Session {
    pub users: Vec<SessionElement>,
    pub now_user: Option<String>,
    pub last_access_time: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            now_user: None,
            last_access_time: chrono::Utc::now().timestamp() as u64,
        }
    }

    pub fn get_mut_user(&mut self, user_id: &str) -> Option<&mut SessionElement> {
        self.users.iter_mut().find(|user| user.user_id == user_id)
    }

    pub fn del_user(&mut self, user_id: &str) {
        self.users.retain(|user| user.user_id != user_id);
    }
}

pub struct SessionElement {
    pub user_id: String,
    pub last_auth_time: u64,
    pub last_access_time: u64,
    pub cache_password: String,
}

impl SessionElement {
    pub fn new(user_id: String, last_auth_time: u64, last_access_time: u64, cache_password: String) -> Self {
        Self {
            user_id,
            last_auth_time,
            last_access_time,
            cache_password,
        }
    }
}
