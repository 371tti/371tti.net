use kurosabi::Kurosabi;
use wk_371tti_net::api::handler::auth::AuthAPI;
use wk_371tti_net::api::handler::search::SearchAPI;
use wk_371tti_net::page_generator::err_page::ErrPage;
use wk_371tti_net::page_generator::search_page::SearchPage;
use wk_371tti_net::context::SiteContext;

pub const CONFIG_PATH: &str = "config.toml";

#[tokio::main(flavor = "multi_thread", worker_threads = 32)]
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
    kurosabi.get("/tool/math_synthesizer", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/math_synthesizer.html")); c });
    kurosabi.get("/tool/2d_code", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/2d_code.html")); c });
    kurosabi.get("/tool/show", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/SHOW.html")); c });
    kurosabi.get("/tool/utf-8_steganography", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/utf-8_steganography.html")); c });
    kurosabi.get("/tool/image_effector", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/image_effector.html")); c });
    kurosabi.get("/game/speed_runner", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/games/speed_runner.html")); c });
    kurosabi.get("/library/mandelbrot", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/mandelbrot.html")); c });
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
    kurosabi.get("/banner.gif", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/banner.gif"), "image/gif"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/favicon.ico", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/favicon.ico"), "image/x-icon"); c });
    kurosabi.get("/robots.txt", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/robots.txt"), "text/plain"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/manifest.json", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/manifest.json"), "application/manifest+json"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    kurosabi.get("/ref", |mut c| async move { c.res.set_status(302); c.res.header.set("Location", "/"); c });
    kurosabi.get("/search/index/add/urls", |mut c| async move { c.res.html(include_str!("../data/pages/search/search_index_add.html")); c });
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
    kurosabi.get("/teapot", |c| async move { ErrPage::status_page(c, 418, "") });
    kurosabi.get("/thisisfine", |c| async move { ErrPage::status_page(c, 218, "") });
    kurosabi.get("/777", |c| async move { ErrPage::status_page(c, 777, "") });
    kurosabi.get("/search", |c| async move { SearchPage::page(c).await });
    kurosabi.get("/api/session", |c| async move { AuthAPI::session(c).await });
    kurosabi.get("/api/search", |c| async move { SearchAPI::search(c).await });
    kurosabi.post("/api/auth", |c| async move { AuthAPI::auth(c).await });
    kurosabi.post("/api/index", |c| async move { SearchAPI::index(c).await });
    kurosabi.not_found_handler(|c| async move { ErrPage::status_page(c, 404, "") });

    let server = kurosabi.server()
        .host([0, 0, 0, 0])
        .accept_threads(1)
        .port(85)
        .thread(16)
        .queue_size(1000)
        .build();

    server.run_async().await;
}