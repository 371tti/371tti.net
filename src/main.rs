use kurosabi::Kurosabi;
use log::debug;
use wk_371tti_net::{api::schema::LoginReq, context::SiteContext};
use wk_371tti_net::api::schema::IndexReq;

pub const CONFIG_PATH: &str = "config.toml";

#[tokio::main]
async fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let context = SiteContext::new(CONFIG_PATH.into()).await;
    let mut kurosabi = Kurosabi::with_context(context);

    kurosabi.get("/", |mut c| async move {
        if c.req.header.get_user_agent().map_or(false, |ua| ua.contains("curl")) {
            c.res.text(include_str!("../data/pages/index/index.curl.txt"));
        } else {
            c.res.html(include_str!("../data/pages/index/index.html"));
        }
        c
    });

    kurosabi.get("/terms", |mut c| async move {c.res.html(include_str!("../data/pages/index/terms/index.html"));c});
    kurosabi.get("/license", |mut c| async move { c.res.html(include_str!("../data/pages/index/license/index.html")); c });
    kurosabi.get("/tools", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/index.html")); c });
    kurosabi.get("/tool/clock", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/clock.html")); c });
    kurosabi.get("/tool/color", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/color.html")); c });
    kurosabi.get("/tool/string_converter", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/string_converter.html")); c });
    kurosabi.get("/tool/music_chord", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/music_chord.html")); c });
    kurosabi.get("/game/speed_runner", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/games/speed_runner.html")); c });
    kurosabi.get("/login", |mut c| async move { c.res.html(include_str!("../data/pages/index/login/index.html")); c });
    kurosabi.get("/release", |mut c| async move { c.res.html(include_str!("../data/pages/index/release/index.html")); c });
    kurosabi.get("/index.html", |mut c| async move { c.res.html(include_str!("../data/pages/index/index.html")); c });
    kurosabi.get("/index", |mut c| async move { c.res.html(include_str!("../data/pages/index/index.html")); c });
    kurosabi.get("/menu.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/menu.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/style.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/style.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/box-load-anime.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/box-load-anime.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/modern-border.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/modern-border.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/modern-border.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/modern-border.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/tag.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/tag.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/copyable.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/copyable.js")); c });
    kurosabi.get("/load-screen.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/load-screen.js")); c });
    kurosabi.get("/371tti_icon.png", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/371tti_icon.png"), "image/png"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/banner.png", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/banner.png"), "image/png"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/favicon.ico", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/favicon.ico"), "image/x-icon"); c });
    kurosabi.get("/robots.txt", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/robots.txt"), "text/plain"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/manifest.json", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/manifest.json"), "application/manifest+json"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/ref", |mut c| async move { c.res.set_status(302); c.res.header.set("Location", "/"); c });
    kurosabi.get("/teapot", |mut c| async move { c.res.set_status(418); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });
    kurosabi.get("/thisisfine", |mut c| async move { c.res.set_status(218); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });
    kurosabi.get("/777", |mut c| async move { c.res.set_status(777); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });
    kurosabi.get("/search", |mut c| async move {
        match c.c.ssr.search_page.generate_search_page(&c).await {
            Ok(html) => {c.res.html(&html);c}
            Err(status) => {c.res.set_status(status);c.res.html(&c.c.ssr.err_page.generate_status_page(&c));c}
        }
    });
    kurosabi.get("/cat", |mut c| async move {
        c.res.text(
r#"
   /\_/\
  ( o.o )
   > ^ <

love cat
"#,
    );
        c
    });

    kurosabi.get("/api/session", |mut c| async move {
        if let Some(session_state) = c.c.req_session_state() {
            let serded_state = serde_json::to_value(&session_state).unwrap_or(serde_json::json!({"error": "Failed to serialize session state"}));
            c.res.json_value(&serded_state);
        } else {
            c.res.set_status(401);
            c.res.json_value(&serde_json::json!({"error": "No valid session"}));
        }
        c
    });

    kurosabi.post("/api/auth", |mut c| async move {
        match c.req.body_de_struct::<LoginReq>().await {
            Ok(login_req) => {
                match c.c.req_login(login_req).await {
                    Ok(res) => {
                        c.res.json_value(&serde_json::to_value(&res).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                    },
                    Err((res, status)) => {
                        c.res.set_status(status);
                        c.res.json_value(&serde_json::to_value(&res).unwrap_or(serde_json::json!({"error": "Failed to serialize response"})));
                    },
                }
                c
            },
            Err(e) => {
                c.res.set_status(400);
                c.res.json_value(&serde_json::json!({"error": format!("Failed to parse request body: {}", e)}));
                return c;
            },
        }
    });

    // Proxy: /api/search -> http://127.0.0.1:90/search (simple pass-through)
    kurosabi.get("/api/search", |mut c| async move {
        let qs = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        let url = if qs.is_empty() {
            "http://127.0.0.1:90/search".to_string()
        } else {
            format!("http://127.0.0.1:90/search?{}", qs)
        };

        match reqwest::get(&url).await {
            Ok(resp) => {
                c.res.set_status(resp.status().as_u16());
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                match resp.bytes().await {
                    Ok(bytes) => { c.res.data(&bytes, &ct); }
                    Err(_) => { c.res.set_status(502); c.res.text("Bad Gateway"); }
                }
            }
            Err(_) => { c.res.set_status(502); c.res.text("Bad Gateway"); }
        }
        c
    });

    kurosabi.post("/api/index", |mut c| async move {

        // Deserialize the request body into IndexReq
        let index_req = match c.req.body_de_struct::<IndexReq>().await {
            Ok(req) => req,
            Err(e) => {
                c.res.text("Bad Request: Failed to parse request body");
                c.res.set_status(400);
                return c;
            }
        };

        // Proxy the request to the backend API
        let url = "http://127.0.0.1:90/add";

        let client = reqwest::Client::new();
        match client
            .post(url)
            .json(&index_req)
            .send()
            .await
        {
            Ok(resp) => {
                c.res.set_status(resp.status().as_u16());
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "application/json".to_string());
                match resp.bytes().await {
                    Ok(bytes) => { c.res.data(&bytes, &ct); }
                    Err(_) => { c.res.set_status(502); c.res.text("Bad Gateway"); }
                }
            }
            Err(_) => { c.res.set_status(502); c.res.text("Bad Gateway"); }
        }

        c
    });

    kurosabi.get("/search/index/add/urls", |mut c| async move {
        c.res.html(include_str!("../data/pages/search/search_index_add.html"));
        c
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.code = 404;
        c.res.html(&c.c.ssr.err_page.generate_status_page(&c));
        c
    });

    let server = kurosabi.server()
        .host([0, 0, 0, 0])
        .accept_threads(1)
        .port(85)
        .thread(8)
        .queue_size(1000)
        .build();

    server.run_async().await;
}