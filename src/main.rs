
use kurosabi::Kurosabi;
use wk_371tti_net::context::SiteContext;

pub const CONFIG_PATH: &str = "config.toml";

fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let context = SiteContext::new(CONFIG_PATH.into());
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
    kurosabi.get("/ref", |mut c| async move { c.res.set_status(302); c.res.header.set("Location", "/"); c });
    kurosabi.get("/teapot", |mut c| async move { c.res.set_status(418); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });
    kurosabi.get("/thisisfine", |mut c| async move { c.res.set_status(218); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });
    kurosabi.get("/777", |mut c| async move { c.res.set_status(777); c.res.html(&c.c.ssr.err_page.generate_status_page(&c)); c });

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


    kurosabi.not_found_handler(|mut c| async move {
        c.res.code = 404;
        c.res.html(&c.c.ssr.err_page.generate_status_page(&c));
        c
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .accept_threads(1)
        .port(85)
        .thread(8)
        .queue_size(1000)
        .build();

    server.run();
}