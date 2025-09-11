use crate::{api::schema::{LoginReq, LoginRes, SessionState}, context::SiteContext};

pub mod schema;

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

    // pub fn req_login(&self, req_json: LoginReq) -> Result<LoginRes, (LoginReq, u16)> {
    //     if req_json.skip_password {

    //     } else {
    //         let auth_res = self.auth.verify_account(
    //             &req_json.account_id, 
    //             &req_json.password
    //         );
    //     }

    //     Ok(LoginRes {
    //         success: true,
    //         message: "Login successful".to_string(),
    //         redirect_to: None,
    //     })
    // }
}