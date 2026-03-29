use crate::{markdown::PageMeta, web::context::SiteContextShared};
use gray_matter::{Matter, engine::YAML};

#[derive(Clone, Default)]
pub struct TemplateService;

impl TemplateService {
    pub fn render_common_page(article_html: String, meta: PageMeta, s_ctx: &SiteContextShared) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        let tags = meta
            .tags()
            .iter()
            .map(|tag| format!("<code>{}</code>", tag))
            .collect::<Vec<_>>()
            .join(", ");
        // let md = if meta.is_complete {
        //     md
        // } else if meta.parse_err.is_none() {
        //     format!(
        //         ">[!Warning] This article is incomplete and may be subject to changes.\n\n{}",
        //         md
        //     )
        // } else {
        //     format!(
        //         ">[!Warning] There was an error parsing the front matter: {}\n\n{}",
        //         meta.parse_err.unwrap_or_default(),
        //         md
        //     )
        // };
        format!(
            include_str!("../../static/index.html"),
            title = title,
            authors = authors,
            description = description,
            content = article_html,
            version = s_ctx.system_info.load().text(),
            count = s_ctx.storage.counter.text_report(),
            tags = tags
        )
    }

    pub fn render_common_html(html: String, meta: PageMeta, s_ctx: &SiteContextShared) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        let tags = meta
            .tags()
            .iter()
            .map(|tag| format!("<code>{}</code>", tag))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            include_str!("../../static/temp.html"),
            title = title,
            authors = authors,
            description = description,
            content = html,
            version = s_ctx.system_info.load().text(),
            count = s_ctx.storage.counter.text_report(),
            tags = tags
        )
    }

    pub fn render_temp_html(html: String, s_ctx: &SiteContextShared) -> String {
        let (meta, content_html) = Self::parse_front_matter(html, &[]);
        TemplateService::render_common_html(content_html, meta, s_ctx)
    }

    pub fn parse_front_matter(md: String, path: &[&str]) -> (PageMeta, String) {
        let matter = Matter::<YAML>::new();
        let result = matter.parse::<PageMeta>(&md);
        match result {
            Ok(data) => {
                if let Some(meta) = data.data {
                    (meta, data.content)
                } else {
                    (
                        PageMeta {
                            title: Some(path.last().unwrap_or(&"Untitled").to_string()),
                            is_complete: true,
                            ..Default::default()
                        },
                        data.content,
                    )
                }
            }
            Err(e) => (
                PageMeta {
                    parse_err: Some(format!("Failed to parse front matter: {}", e)),
                    title: Some(path.last().unwrap_or(&"Untitled").to_string()),
                    is_complete: false,
                    ..Default::default()
                },
                md,
            ),
        }
    }
}
