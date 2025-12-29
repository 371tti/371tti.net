use kurosabi::kurosabi::Context;
use tokio::io::{AsyncWriteExt, duplex};

use crate::{context::SiteContext, features::curl_chat::CurlChatPacket, utils::url_decode};

pub struct CurlChat;

impl CurlChat {
    const BUF_SIZE: usize = 256;
    const COOKIE_MESSAGE: &str = r#"
このAPIは Cookie によるセッション管理を行います。

curl に対して 環境遠陬変数 CURL_COOKIE_JAR を設定し、てください。
またこれはグローバル設定なのできおつけてくださいね。

Linux / macOS:
```
# 初期設定
mkdir -p "$HOME/.curl"
COOKIE="$HOME/.curl/cookies.txt"

# 以後のアクセスで Cookie を保存・送信
curl -b "$COOKIE" -c "$COOKIE" https://dev.371tti.net/cc
```

Windows (PowerShell):
```
# PSの場合これで永続化
mkdir $env:USERPROFILE\.curl -Force
@"
cookie-jar = $env:USERPROFILE\.curl\cookies.txt
cookie = $env:USERPROFILE\.curl\cookies.txt
"@ | Set-Content -Encoding ascii $env:USERPROFILE\.curlrc

# 以後のアクセスで Cookie を保存・送信
curl https://dev.371tti.net/cc
```

設定後、もう2度アクセスしてください。
"#;
    const VERIFIED_MESSAGE: &str = "セッションを確認しました。/cc/send/<msg> でメッセージを送信できます \n";
    const UNVERIFIED_MESSAGE: &str = "メッセージを送信するには /cc/send/<msg> \n";

    const NAV_SET_USERNAME: &str = r#"
ユーザー名を設定していません。/cc/set_name/<username> で設定してください
"#;
    const KEY_NAME: &str = "curl_chat_username";

    pub async fn root(mut c: Context<SiteContext>) -> Context<SiteContext> {
        let mut chat_rx = c.c.curl_chat_state.broadcast_rx.resubscribe();
        let (mut a, b) = duplex(Self::BUF_SIZE);
        let first_msg = if c.c.new_session_connected {
            Self::VERIFIED_MESSAGE
        } else {
            Self::UNVERIFIED_MESSAGE
        };
        let _ = a.write_all(first_msg.as_bytes()).await;
        let _ = a.flush().await;
        tokio::spawn(async move {
            while let Ok(packet) = chat_rx.recv().await {
                let msg = format!("\x1b[2K\r{}: {}\n", packet.username, packet.message);
                if a.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if a.flush().await.is_err() {
                    break;
                }
            }
        });
        c.res.header.set("Content-Type", "text/plain; charset=utf-8");
        c.res.header.set("X-Accel-Buffering", "no");
        c.res.header.set("Cache-Control", "no-cache, no-transform");
        c.res.chunked_stream(Box::pin(b), Self::BUF_SIZE);
        c
    }

    pub async fn send_msg(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if c.c.new_session_connected {
            c.res.text(Self::COOKIE_MESSAGE);
            return c;
        }

        let session_key = match c.c.session_key.clone() {
            Some(key) => key,
            None => {
                c.res.text("Unauthorized: No valid session.\n");
                c.res.set_status(401);
                return c;
            }
        };

        enum UserNameLookup {
            Ok(String),
            NeedSetName,
            Unauthorized,
        }

        let user_name_lookup = {
            let sessions = &c.c.auth.sessions;
            match sessions.get(&session_key) {
                Some(session_data) => {
                    if let Some(name) = session_data.data_storage.get(Self::KEY_NAME) {
                        UserNameLookup::Ok(name.clone())
                    } else {
                        UserNameLookup::NeedSetName
                    }
                }
                None => UserNameLookup::Unauthorized,
            }
        };

        let user_name = match user_name_lookup {
            UserNameLookup::Ok(name) => name,
            UserNameLookup::NeedSetName => {
                c.res.text(Self::NAV_SET_USERNAME);
                return c;
            }
            UserNameLookup::Unauthorized => {
                c.res.text("Unauthorized: No valid session.\n");
                c.res.set_status(401);
                return c;
            }
        };

        let base = c.req.path.get_field("*").unwrap_or("".into());
        let msg = match url_decode(&base) {
            Ok(s) => s,
            Err(_) => {
                c.res.text("Bad Request: Failed to decode message.\n");
                c.res.set_status(400);
                return c;
            }
        };

        let packet = CurlChatPacket {
            username: user_name,
            message: msg,
            session_key: session_key.clone(),
        };


        let _ = c.c.curl_chat_state.broadcast_tx.send(packet);
        c.res.text("メッセージを送信しました。\n");
        c
    }

    pub async fn set_name(mut c: Context<SiteContext>) -> Context<SiteContext> {
        if c.c.new_session_connected {
            c.res.text(Self::COOKIE_MESSAGE);
            return c;
        }

        let session_key = match c.c.session_key.clone() {
            Some(key) => key,
            None => {
                c.res.text("Unauthorized: No valid session.\n");
                c.res.set_status(401);
                return c;
            }
        };

        let base = c.req.path.get_field("*").unwrap_or("".into());
        let name = match url_decode(&base) {
            Ok(s) => s,
            Err(_) => {
                c.res.text("Bad Request: Failed to decode username.\n");
                c.res.set_status(400);
                return c;
            }
        };

        c.c.auth.sessions.get_mut(&session_key).map(|mut session_data| {
            session_data.data_storage.insert(Self::KEY_NAME.into(), name.clone());
        });

        c.res.text(&format!("ユーザー名を '{}' に設定しました。\n", name));
        c
    }
}