use std::{path::Path, sync::Arc};

use kurosabi::{
    connection::file::{DirEntryInfo, FileContentBuilder},
    utils::{url_decode_fast, url_encode},
};
use tokio::io::AsyncReadExt;

use crate::{
    config::Config, markdown::PageMeta, web::{context::SiteContextShared, templates::TemplateService}
};

#[derive(Clone)]
pub struct DocsRouter {
    config: Arc<Config>,
}

impl DocsRouter {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn route(
        &self,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> std::io::Result<Option<String>> {
        let builder = FileContentBuilder::base(&self.config.base_dir)
            .path_url_segs(path)
            .check_file_exists()
            .await;

        let builder = match builder {
            Ok(found) => found,
            // if it's a directory
            Err(Some(dir)) => return Ok(Some(self.render_dir(dir, path, s_ctx).await?)),
            Err(None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ));
            }
        };

        let mut file = builder.build().await?;
        match classify_mime(&file.mime_type, &file.file_path) {
            DocKind::Markdown => {
                let mut buf = String::new();
                let _bytes = file.file.read_to_string(&mut buf).await?;
                let (meta, content_md) = TemplateService::parse_front_matter(buf, path);
                let html = TemplateService::render_common_page(content_md, meta, s_ctx);
                Ok(Some(html))
            }
            DocKind::Html => {
                let mut buf = String::new();
                let _bytes = file.file.read_to_string(&mut buf).await?;
                let (meta, content_html) = TemplateService::parse_front_matter(buf, path);
                let html = TemplateService::render_common_html(content_html, meta, s_ctx);
                Ok(Some(html))
            }
            DocKind::Other => Ok(None),
        }
    }

    async fn render_dir(
        &self,
        dir: Vec<DirEntryInfo>,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> std::io::Result<String> {
        let mut path_with_index = if path == [""] { vec![] } else { path.to_vec() };
        if dir.iter().any(|e| {
            e.kind.is_file() && e.path.file_name().and_then(|n| n.to_str()) == Some("index.html")
        }) {
            path_with_index.push("index.html");
            match FileContentBuilder::base(&self.config.base_dir)
                .path_url_segs(&path_with_index)
                .build()
                .await
            {
                Ok(mut file) => {
                    if !file.mime_type.contains("text/html;") {
                        path_with_index.pop();
                    } else {
                        let mut buf = String::new();
                        let _ = file.file.read_to_string(&mut buf).await;
                        let (meta, content_html) = TemplateService::parse_front_matter(buf, path);
                        return Ok(TemplateService::render_common_html(
                            content_html,
                            meta,
                            s_ctx,
                        ));
                    }
                }
                Err(_) => {
                    let _ = path_with_index.pop();
                }
            };
        }
        path_with_index.push("index.md");
        let index_md: Option<String> = match FileContentBuilder::base(&self.config.base_dir)
            .path_url_segs(&path_with_index)
            .build()
            .await
        {
            Ok(mut file) => {
                let mut buf = String::new();
                file.file.read_to_string(&mut buf).await?;
                Some(buf)
            }
            Err(_) => None,
        };
        let mut files: Vec<&str> = Vec::new();
        let mut dirs: Vec<&str> = Vec::new();
        for entry in dir.iter() {
            if entry.kind.is_dir() {
                let opt_dir_name = entry.path.file_name().and_then(|n| n.to_str());
                if let Some(dir_name) = opt_dir_name
                    && !dir_name.starts_with(".")
                {
                    dirs.push(dir_name);
                }
            } else if entry.kind.is_file() {
                let opt_file_name = entry.path.file_name().and_then(|n| n.to_str());
                if let Some(file_name) = opt_file_name
                    && !file_name.starts_with(".")
                    && file_name != "index.md"
                {
                    files.push(file_name);
                }
            }
        }
        files.sort_unstable();
        dirs.sort_unstable();
        let path_segments: Vec<String> = path
            .iter()
            .filter(|p| !p.is_empty())
            .map(|s| url_decode_fast(s).to_string())
            .collect();
        let encoded_path = path_segments
            .iter()
            .map(|segment| url_encode(segment))
            .collect::<Vec<_>>()
            .join("/");

        let (meta, opt_md) = match &index_md {
            Some(md) => {
                let (m, c) = TemplateService::parse_front_matter(md.clone(), path);
                (m, Some(c))
            }
            None => (
                PageMeta {
                    title: Some(path_segments.last().map_or("Root", |v| v).to_string()),
                    authors: Some(vec!["system".to_string()]),
                    is_complete: true,
                    ..Default::default()
                },
                None,
            ),
        };
        let mut md = String::new();
        md.push_str(&format!(
            "## Index of [root](/)/{}",
            path_segments
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == path_segments.len() - 1 {
                        p.to_string()
                    } else {
                        let href = format!(
                            "/{}",
                            path_segments
                                .iter()
                                .take(i + 1)
                                .map(|segment| url_encode(segment))
                                .collect::<Vec<_>>()
                                .join("/")
                        );
                        format!("[{}]({})", p, href)
                    }
                })
                .collect::<Vec<_>>()
                .join("/")
        ));
        md.push_str(&match dirs
            .iter()
            .map(|d| {
                format!(
                    "- [{}]({}/{})",
                    d,
                    if encoded_path.is_empty() {
                        "".to_string()
                    } else {
                        format!("/{}", encoded_path)
                    },
                    url_encode(d)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
        {
            s if s.is_empty() => "".to_string(),
            s => format!("\n### Directories\n{}", s),
        });
        md.push_str(&match files
            .iter()
            .map(|f| {
                format!(
                    "- [{}]({}/{})",
                    f,
                    if encoded_path.is_empty() {
                        "".to_string()
                    } else {
                        format!("/{}", encoded_path)
                    },
                    url_encode(f)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
        {
            s if s.is_empty() => "".to_string(),
            s => format!("\n### Files\n{}", s),
        });
        md.push_str(&match opt_md {
            Some(content) => format!("\n\n---\n\n{}", content),
            None => "".to_string(),
        });
        Ok(TemplateService::render_common_page(md, meta, s_ctx))
    }
}

enum DocKind {
    Markdown,
    Html,
    Other,
}

fn classify_mime(mime_type: &str, path: &Path) -> DocKind {
    let file_ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    if file_ext == "md" || file_ext == "markdown" {
        return DocKind::Markdown;
    }
    if file_ext == "html" || file_ext == "htm" {
        return DocKind::Html;
    }
    if mime_type.contains("text/markdown") {
        return DocKind::Markdown;
    }
    if mime_type.contains("text/html") {
        return DocKind::Html;
    }
    DocKind::Other
}
