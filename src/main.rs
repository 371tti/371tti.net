
use kurosabi::Kurosabi;
use wk_371tti_net::context::SiteContext;



#[tokio::main]
async fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let context = SiteContext::new();
    let mut kurosabi = Kurosabi::with_context(context);

    kurosabi.get("/", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/index.html"));
        Ok(c)
    });

    kurosabi.get("/terms", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/terms/index.html"));
        Ok(c)
    });

    kurosabi.get("/license", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/license/index.html"));
        Ok(c)
    });

    kurosabi.get("/login", |mut c| async move {
        c.c.init(&mut c.req, &mut c.res);
        c.res.html(include_str!("../data/pages/index/login/index.html"));
        Ok(c)
    });

    kurosabi.get("/index.html", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/index.html"));
        Ok(c)
    });

    kurosabi.get("/index", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/index.html"));
        Ok(c)
    });

    kurosabi.get("/style.css", |mut c| async move {
        c.res.css(include_str!("../data/pages/index/style.css"));
        Ok(c)
    });

    kurosabi.get("/box-load-anime.js", |mut c| async move {
        c.res.js(include_str!("../data/pages/index/box-load-anime.js"));
        Ok(c)
    });

    
    kurosabi.get("/modern-border.css", |mut c| async move {
        c.res.css(include_str!("../data/pages/index/modern-border.css"));
        Ok(c)
    });

    kurosabi.get("/modern-border.js", |mut c| async move {
        c.res.js(include_str!("../data/pages/index/modern-border.js"));
        Ok(c)
    });

    kurosabi.get("/tag.css", |mut c| async move {
        c.res.css(include_str!("../data/pages/index/tag.css"));
        Ok(c)
    });

    kurosabi.get("/copyable.js", |mut c| async move {
        c.res.js(include_str!("../data/pages/index/copyable.js"));
        Ok(c)
    });

    kurosabi.get("/371tti_icon.png", |mut c| async move {
        c.res.data(include_bytes!("../data/pages/index/371tti_icon.png"), "image/png");
        Ok(c)
    });

    kurosabi.get("/banner.png", |mut c| async move {
        c.res.data(include_bytes!("../data/pages/index/banner.png"), "image/png");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        Ok(c)
    });

    kurosabi.get("/favicon.ico", |mut c| async move {
        c.res.data(include_bytes!("../data/pages/index/favicon.ico"), "image/x-icon");
        Ok(c)
    });

    kurosabi.get("/*", |mut c| async move {
        c.res.code = 404;
        c.res.html(&c.c.ssr.err_page.generate_status_page(&c));
        Ok(c)
    });

    kurosabi.get("/api/session-status", |mut c| async move {
        c.c.init(&mut c.req, &mut c.res);
        let session_status = c.c.aurth_manager.api_session_status(&c.c.session_id).unwrap();
        c.res.json_value(&serde_json::json!(session_status));
        Ok(c)
    });

    kurosabi.post("/api/login", |mut c| async move {
        c.c.init(&mut c.req, &mut c.res);
        let value = c.req.body_json().await.unwrap();
        Ok(c)
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(85)
        .thread(8)
        .queue_size(1000)
        .build();

    server.run().await;
}