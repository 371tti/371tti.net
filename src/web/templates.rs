use kurosabi::utils::{url_decode_fast, url_encode};

use crate::{DOMAIN, markdown::PageMeta, web::context::SiteContextShared};
use gray_matter::{Matter, engine::YAML};

#[derive(Clone, Default)]
pub struct TemplateService;

impl TemplateService {
    pub fn render_common_page(
        article_html: String,
        meta: PageMeta,
        path: &[&str],
        request_path: &str,
        s_ctx: &SiteContextShared,
    ) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        let og_image_url = Self::build_thumbnail_url(path);
        let og_url = Self::escape_html_attr(&Self::build_page_url(request_path));
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
            domain = DOMAIN,
            title = title,
            authors = authors,
            description = description,
            content = article_html,
            version = s_ctx.system_info.load().text(),
            count = s_ctx.storage.counter.text_report(),
            tags = tags,
            og_image_url = og_image_url,
            og_url = og_url
        )
    }

    pub fn render_common_html(
        html: String,
        meta: PageMeta,
        path: &[&str],
        request_path: &str,
        s_ctx: &SiteContextShared,
    ) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        let og_image_url = Self::build_thumbnail_url(path);
        let og_url = Self::escape_html_attr(&Self::build_page_url(request_path));
        let tags = meta
            .tags()
            .iter()
            .map(|tag| format!("<code>{}</code>", tag))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            include_str!("../../static/temp.html"),
            domain = DOMAIN,
            title = title,
            authors = authors,
            description = description,
            content = html,
            version = s_ctx.system_info.load().text(),
            count = s_ctx.storage.counter.text_report(),
            tags = tags,
            og_image_url = og_image_url,
            og_url = og_url
        )
    }

    pub fn render_temp_html(html: String, request_path: &str, s_ctx: &SiteContextShared) -> String {
        let (meta, content_html) = Self::parse_front_matter(html, &[]);
        TemplateService::render_common_html(content_html, meta, &[], request_path, s_ctx)
    }

    pub fn build_thumbnail_url(path: &[&str]) -> String {
        let mut url = String::from("/api/thumbnail");
        let mut has_segment = false;

        for segment in path.iter().copied().filter(|segment| !segment.is_empty()) {
            let decoded = url_decode_fast(segment);
            has_segment = true;
            url.push('/');
            url.push_str(&url_encode(&decoded));
        }

        if !has_segment {
            url.push('/');
        }

        url
    }

    fn build_page_url(request_path: &str) -> String {
        if request_path.starts_with('/') {
            format!("https://{}{}", DOMAIN, request_path)
        } else {
            format!("https://{}/{}", DOMAIN, request_path)
        }
    }

    fn escape_html_attr(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
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
