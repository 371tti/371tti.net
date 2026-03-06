use kurosabi::{
    connection::file::FileContentBuilder,
    http::{HttpMethod, HttpStatusCode},
    server::tokio::KurosabiTokioServerBuilder,
};
use wk_371tti_net::{SESSION_COOKIE_NAME, web::SiteContext};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wk_371tti_net::updater=debug"),
    )
    .format_timestamp_millis()
    .init();
    wk_371tti_net::print_logo();
    let context = SiteContext::new().await?;
    let config = &context.shared.clone().config;
    let builder: KurosabiTokioServerBuilder<SiteContext> =
        KurosabiTokioServerBuilder::with_context(context);
    builder
        .bind(config.get_host())
        .port(config.http_config.port)
        .router_and_build(|mut conn| async move {
            let conn = if let Some(cookie) = conn
                .c
                .session_check(conn.req.get_cookie(SESSION_COOKIE_NAME).await.as_deref())
            {
                conn.set_cookie(cookie)
            } else {
                conn
            };
            let conn = match conn.req.method() {
                HttpMethod::GET => match conn.path_segs().as_ref() {
                    ["robots.txt"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .text_body(include_str!("../static/robots.txt")),
                    [".well-known", "security.txt"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .text_body(include_str!("../static/security.txt")),
                    ["banner.gif"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .add_header("Content-Type", "image/gif")
                        .binary_body(include_bytes!("../static/banner.gif")),
                    ["banner.png"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .add_header("Content-Type", "image/png")
                        .binary_body(include_bytes!("../static/banner.png")),
                    ["menu.js"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .js_body(include_str!("../static/menu.js")),
                    ["style.css"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .css_body(include_str!("../static/style.css")),
                    ["code-tool.js"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .js_body(include_str!("../static/code-tool.js")),
                    ["optimizer.js"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .js_body(include_str!("../static/optimizer.js")),
                    ["load-screen.js"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .js_body(include_str!("../static/load-screen.js")),
                    ["manifest.json"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .json_body(include_str!("../static/manifest.json")),
                    ["favicon.ico"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .add_header("Content-Type", "image/x-icon")
                        .binary_body(include_bytes!("../static/favicon.ico")),
                    ["icon.png"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .png_body(include_bytes!("../static/icon.png")),
                    ["371tti_icon.png"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .png_body(include_bytes!("../static/371tti_icon.png")),
                    ["ls", path @ ..] => match conn.c.ls_routing(path).await {
                        Ok(result) => match conn.json_body_serialized(&result) {
                            Ok(c) => c,
                            Err(e) => e
                                .connection
                                .set_status_code(HttpStatusCode::InternalServerError)
                                .no_body(),
                        },
                        Err(_) => conn
                            .set_status_code(HttpStatusCode::InternalServerError)
                            .no_body(),
                    },
                    ["api", "session"] => {
                        conn.text_body("not impl")
                    },
                    ["raw", path @ ..] => {
                        let content = FileContentBuilder::base(&conn.c.shared.config.base_dir)
                            .path_url_segs(path)
                            .inline();
                        if path.first() == Some(&"static") {
                            conn.add_header("Cache-Control", "public, max-age=300, must-revalidate")
                                .file_body(content)
                                .await
                                .unwrap_or_else(|p| p.connection)
                        } else {
                            conn.file_body(content)
                                .await
                                .unwrap_or_else(|p| p.connection)
                        }
                    }
                    path => match conn.c.docs_routing(path).await {
                        Ok(Some(html)) => {
                            conn.c.shared.storage.counter.increment_page_views();
                            conn.html_body(html)
                        }
                        Ok(None) => {
                            let redirect_path = "/raw/".to_string() + &path.join("/");
                            conn.redirect(redirect_path)
                        }
                        Err(_) => conn.set_status_code(HttpStatusCode::NotFound).no_body(),
                    },
                },
                _ => conn
                    .set_status_code(HttpStatusCode::MethodNotAllowed)
                    .no_body(),
            };
            let status = conn.res.status_code().into();
            conn.c.shared.storage.counter.increment(status);
            // 新しいセッションで、かつ成功レスポンスの場合はユニークビジター数を増やす～～
            if conn.c.new_session && status < 400 {
                conn.c.shared.storage.counter.increment_unique_visitors();
            }
            if status == 404 {
                let not_found_html = conn.c.not_found_routing();
                conn.cancel()
                    .set_status_code(HttpStatusCode::NotFound)
                    .html_body(not_found_html)
            } else {
                conn
            }
        })
        .run()
        .await
}
