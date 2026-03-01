use gray_matter::{Matter, engine::YAML};
use crate::{VERSION, markdown::PageMeta, render::MarkdownRenderer};

#[derive(Clone)]
pub struct TemplateService {
    renderer: MarkdownRenderer,
}

impl Default for TemplateService {
    fn default() -> Self {
        Self {
            renderer: MarkdownRenderer::default(),
        }
    }
}

impl TemplateService {
    pub fn render_common_page(&self, md: String, meta: PageMeta) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        let md = if meta.is_complete {
            md
        } else if meta.parse_err.is_none() {
            format!(
                ">[!Warning] This article is incomplete and may be subject to changes.\n\n{}",
                md
            )
        } else {
            format!(
                ">[!Warning] There was an error parsing the front matter: {}\n\n{}",
                meta.parse_err.unwrap_or_default(),
                md
            )
        };
        let content = self.renderer.render(&md);
        format!(
            include_str!("../../data/static/index.html"),
            title = title,
            authors = authors,
            description = description,
            content = content,
            version = VERSION
        )
    }

    pub fn render_common_html(&self, html: String, meta: PageMeta) -> String {
        let title = meta.title();
        let description = meta.description();
        let authors = meta.authors();
        format!(
            include_str!("../../data/static/temp.html"),
            title = title,
            authors = authors,
            description = description,
            content = html,
            version = VERSION
        )
    }

    pub fn parse_front_matter(&self, md: String, path: &[&str]) -> (PageMeta, String) {
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
