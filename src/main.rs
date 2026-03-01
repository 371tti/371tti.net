use kurosabi::{
    connection::file::FileContentBuilder,
    http::{HttpMethod, HttpStatusCode},
    server::tokio::KurosabiTokioServerBuilder,
};
use wk_371tti_net::{config::BASE_DIR, web::SiteContext};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let context = SiteContext::new();
    let builder: KurosabiTokioServerBuilder<SiteContext> =
        KurosabiTokioServerBuilder::with_context(context);
    builder
        .bind([0, 0, 0, 0])
        .port(85)
        .router_and_build(|conn| async move {
            let conn = match conn.req.method() {
                HttpMethod::GET => match conn.path_segs().as_ref() {
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
                    ["raw", path @ ..] => {
                        let content = FileContentBuilder::base(BASE_DIR)
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
                    ["menu.js"] => conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").js_body(include_str!("../data/static/menu.js")),
                    ["style.css"] => conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").css_body(include_str!("../data/static/style.css")),
                    ["code-tool.js"] => conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").js_body(include_str!("../data/static/code-tool.js")),
                    ["optimizer.js"] => conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").js_body(include_str!("../data/static/optimizer.js")),
                    ["load-screen.js"] => {
                        conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").js_body(include_str!("../data/static/load-screen.js"))
                    }
                    ["manifest.json"] => {
                        conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").json_body(include_str!("../data/static/manifest.json"))
                    }
                    ["favicon.ico"] => conn
                        .add_header("Cache-Control", "public, max-age=300, must-revalidate")
                        .add_header("Content-Type", "image/x-icon")
                        .binary_body(include_bytes!("../data/static/favicon.ico")),
                    ["icon.png"] => conn.add_header("Cache-Control", "public, max-age=300, must-revalidate").png_body(include_bytes!("../data/static/icon.png")),
                    [path @ ..] => match conn.c.docs_routing(path).await {
                        Ok(Some(html)) => conn.html_body(html),
                        Ok(None) => {
                            let redirect_path = "/raw/".to_string() + &path.join("/");
                            conn.redirect(redirect_path)
                        }
                        Err(_) => conn.set_status_code(HttpStatusCode::NotFound).no_body(),
                    },
                    _ => conn.set_status_code(HttpStatusCode::NotFound).no_body(),
                },
                _ => conn
                    .set_status_code(HttpStatusCode::MethodNotAllowed)
                    .no_body(),
            };
            let status = conn.res.status_code().into();
            conn.c.shared.counter.increment(status);
            conn
        })
        .run()
        .await
}
