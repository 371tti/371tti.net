use kurosabi::Kurosabi;
use wk_371tti_net::api::handler::aaa::AsciiArcAnimation;
use wk_371tti_net::api::handler::auth::AuthAPI;
use wk_371tti_net::api::handler::search::SearchAPI;
use wk_371tti_net::api::handler::status::StatusAPI;
use wk_371tti_net::page_generator::PageGenerator;
use wk_371tti_net::page_generator::article::ArticlePage;
use wk_371tti_net::page_generator::err::ErrPage;
use wk_371tti_net::page_generator::search::SearchPage;
use wk_371tti_net::context::SiteContext;

pub const CONFIG_PATH: &str = "config.toml";

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());

    let context = SiteContext::new(CONFIG_PATH.into()).await;
    let mut app = Kurosabi::with_context(context);

    app.get("/blog/*", |c| async move { ArticlePage::page(c).await });
    app.get("/blog/", |c| async move { ArticlePage::page(c).await });
    app.get("/blog", |c| async move { ArticlePage::page(c).await });
    app.get("/", |c| async move {PageGenerator::base(c, include_str!("../data/pages/index/index.html"), "Home", None) });

    app.get("/terms", |c| async move {PageGenerator::base(c, include_str!("../data/pages/index/terms/index.html"), "Terms of Service", None)});
    app.get("/license", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/license/index.html"), "License", None) });
    app.get("/tools", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/index.html"), "Tools", None) });
    app.get("/tool/clock", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/clock.html"), "Clock", None) });
    app.get("/tool/color", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/color.html"), "Color", None) });
    app.get("/tool/math_synthesizer", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/math_synthesizer.html"), "Math Synthesizer", None) });
    app.get("/tool/2d_code", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/2d_code.html"), "2D Code", None) });
    app.get("/tool/show", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/SHOW.html"), "Show", None) });
    app.get("/tool/utf-8_steganography", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/utf-8_steganography.html"), "UTF-8 Steganography", None) });
    app.get("/tool/image_effector", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/image_effector.html"), "Image Effector", None) });
    app.get("/game/speed_runner", |mut c| async move { c.res.html(include_str!("../data/pages/index/tools/games/speed_runner.html")); c });
    app.get("/library/mandelbrot", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/tools/mandelbrot.html"), "Mandelbrot", None) });
    app.get("/toy/js_for_hater", |c| async move { PageGenerator::base(c, include_str!("../data/pages/toy/js_for_hater.html"), "JS for Hater", None) });
    app.get("/login", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/login/index.html"), "Login", None) });
    app.get("/release", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/release/index.html"), "Release", None) });
    app.get("/index.html", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/index.html"), "Home", None) });
    app.get("/index", |c| async move { PageGenerator::base(c, include_str!("../data/pages/index/index.html"), "Home", None) });
    app.get("/menu.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/menu.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/optimizer.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/optimizer.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/rw-code.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/rw-code.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/style.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/style.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/page-anime.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/page-anime.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/modern-border.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/modern-border.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/modern-border.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/modern-border.js")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/tag.css", |mut c| async move { c.res.css(include_str!("../data/pages/index/tag.css")); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/copyable.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/copyable.js")); c });
    app.get("/load-screen.js", |mut c| async move { c.res.js(include_str!("../data/pages/index/load-screen.js")); c });
    app.get("/371tti_icon.png", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/371tti_icon.png"), "image/png"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/banner.png", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/banner.png"), "image/png"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/banner.gif", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/banner.gif"), "image/gif"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/favicon.ico", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/favicon.ico"), "image/x-icon"); c });
    app.get("/robots.txt", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/robots.txt"), "text/plain"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/humans.txt", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/humans.txt"), "text/plain"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/security.txt", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/security.txt"), "text/plain"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/manifest.json", |mut c| async move { c.res.data(include_bytes!("../data/pages/index/manifest.json"), "application/manifest+json"); c.res.header.set("Access-Control-Allow-Origin", "*"); c });
    app.get("/status", |c| async move { PageGenerator::base(c, include_str!("../data/pages/status/index.html"), "Status", None) });
    app.get("/hekade" , |c| async move { PageGenerator::base(c, include_str!("../data/pages/hekade/index.html"), "Hekade", None) });
    app.get("/ref", |mut c| async move { c.res.set_status(302); c.res.header.set("Location", "/"); c });
    app.get("/search/index/add/urls", |c| async move { PageGenerator::base(c, include_str!("../data/pages/search/search_index_add.html"), "Search Index Add URLs", None) });
    app.get("/ping", |c| async move { StatusAPI::ping(c).await });
    app.get("/aaa", |c| async move { AsciiArcAnimation::root(c).await });
    app.get("/aaa/kaomoji", |c| async move { AsciiArcAnimation::kaomoji(c).await });
    app.get("/aaa/cat", |c| async move { AsciiArcAnimation::cat(c).await });
    app.get("/aaa/bad_apple", |c| async move { AsciiArcAnimation::bad_apple(c).await });
    app.get("/teapot", |c| async move { ErrPage::status_page(c, 418, "") });
    app.get("/thisisfine", |c| async move { ErrPage::status_page(c, 218, "") });
    app.get("/777", |c| async move { ErrPage::status_page(c, 777, "") });
    app.get("/search", |c| async move { SearchPage::page(c).await });
    app.get("/api/search", |c| async move { SearchAPI::search(c).await });
    app.post("/api/auth", |c| async move { AuthAPI::auth(c).await });
    app.get("/api/session", |c| async move { AuthAPI::session(c).await });
    app.post("/api/index", |c| async move { SearchAPI::index(c).await });
    app.post("/api/meta", |c| async move { SearchAPI::meta(c).await });
    app.get("/api/meta/*", |c| async move { SearchAPI::meta_get(c).await });
    app.post("/api/token_freq", |c| async move { SearchAPI::token_freq(c).await });
    app.get("/api/token_freq/*", |c| async move { SearchAPI::token_freq_get(c).await });
    app.get("/api/status", |c| async move { StatusAPI::health(c).await });
    app.not_found_handler(|c| async move { ErrPage::status_page(c, 404, "") });

    let server = app.server()
        .host([0, 0, 0, 0])
        .port(85)
        .thread(16)
        .nodelay(true)
        .queue_size(1000)
        .build();

    server.run_async().await;
}