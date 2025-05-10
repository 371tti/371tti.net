use std::sync::Arc;

use kurosabi::Kurosabi;
use wk_371tti_net::context::SiteContext;



#[tokio::main]
async fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let arc_context = Arc::new(SiteContext::new());
    let mut kurosabi = Kurosabi::with_context(arc_context);

    kurosabi.get("/", |mut c| async move {
        c.res.html(include_str!("../data/pages/index/index.html"));
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
        Ok(c)
    });


    kurosabi.get("/*", |mut c| async move {
        c.res.code = 404;
        c.res.html(&c.c.ssr.err_page.generate_status_page(&c));
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