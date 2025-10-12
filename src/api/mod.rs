
use chrono::Utc;

use crate::{api::schema::auth::{LoginReq, LoginRes, SessionState}, context::SiteContext, user_manager::auth_manager::{AccountSession, VerifyResult}};

pub mod schema;
pub mod handler;

impl SiteContext {
    pub fn req_session_state(&self) -> Option<SessionState> {
        // Option<SessionKey> -> Option<&SessionKey>
        let session_key = self.session_key.as_ref()?;
        let session = self.auth.sessions.get(session_key)?;
        let is_logged_in = session.now_account_index.is_some();
        let created_at = session.created_at;
        let logged_account = self.user_id.clone();
        let authenticated_accounts = session.accounts.clone();

        Some(SessionState {
            is_logged_in,
            logged_account,
            authenticated_accounts,
            created_at,
        })
    }

    /// Handle login request
    /// パスワードスキップの場合
    /// セッションのアカウントリストにそのアカウントがあるかつ
    /// そのアカウントが有効である
    /// 
    /// パスワードありの場合
    /// 認証を通過したら
    /// セッションのアカウントを更新 重複なら上書き
    /// アカウントのほうにセッションをリンク
    pub async fn req_login(&self, req_json: LoginReq) -> Result<LoginRes, (LoginRes, u16)> {
        if req_json.skip_password {
            // ちょっと無名関数定義
            let mk_err = |status: u16, msg: &str| -> (LoginRes, u16) {
                (
                    LoginRes {
                        success: false,
                        message: msg.to_string(),
                        redirect_to: None,
                    },
                    status
                )
            };
            let now = Utc::now();

            // セッション取得 get lock here 
            let mut session = self.session_key
                .as_ref()
                .and_then(|key| self.auth.sessions.get_mut(key))
                .ok_or_else(|| mk_err(500, "No valid session. Unexpected error, possibly serious."))?;

            // 対象アカウント（インデックス付きで一度に取得）
            let (idx, acc_sess) = session.accounts
                .iter_mut()
                .enumerate()
                .find(|(_, a)| a.get_account_id() == &req_json.account_id)
                .ok_or_else(|| mk_err(403, "Account not in session's authenticated accounts."))?;

            // ログイン状態と期限チェック
            let last_time = acc_sess.get_time()
                .ok_or_else(|| mk_err(403, "Account is logged out."))?;

            if now.signed_duration_since(last_time) > self.auth.account_timeout {
                return Err(mk_err(403, "Account session has timed out."));
            }

            // 現在操作中アカウント更新とアクセス時間更新
            acc_sess.set_time(now);
            session.now_account_index = Some(idx);
            // free lock here
            return Ok(LoginRes {
                success: true,
                message: "Login successful.".to_string(),
                redirect_to: Some(format!("/dashboard/{}", req_json.account_id.as_str())),
            });

        } else {
            let auth_res = self.auth.verify_account(
                &req_json.account_id, 
                &req_json.password
            ).await;
            match auth_res {
                VerifyResult::Success => {
                    let mut session = self.session_key
                        .as_ref()
                        .and_then(|key| self.auth.sessions.get_mut(key))
                        .ok_or_else(|| (LoginRes {
                            success: false,
                            message: "No valid session. Unexpected error, possibly serious.".to_string(),
                            redirect_to: None,
                        }, 500))?;

                    // アカウントが既に存在するか探す
                    let now = Utc::now();
                    let idx = session.accounts.iter_mut().enumerate().find_map(|(i, a)| {
                        if a.get_account_id() == &req_json.account_id {
                            // 見つかった場合はアクセス時間を更新
                            a.set_time(now);
                            Some(i)
                        } else {
                            None
                        }
                    });

                    let idx = match idx {
                        Some(i) => i,
                        None => {
                            let mut id = AccountSession::new(&req_json.account_id);
                            id.set_time(now);
                            self.auth.accounts.link_account_session(&req_json.account_id, session.key()).await;
                            session.push_account(
                                id
                            )
                        }
                    };

                    // 現在操作中アカウント更新
                    session.now_account_index = Some(idx);
                    return Ok(LoginRes {
                        success: true,
                        message: "Login successful.".to_string(),
                        redirect_to: Some(format!("/login")),
                    });
                },
                VerifyResult::NoSuchAccount => {
                    return Err((LoginRes {
                        success: false,
                        message: "No such account.".to_string(),
                        redirect_to: None,
                    }, 403));
                },
                VerifyResult::InvalidPassword => {
                    return Err((LoginRes {
                        success: false,
                        message: "Invalid password.".to_string(),
                        redirect_to: None,
                    }, 403));
                },
            }
        }
    }
}