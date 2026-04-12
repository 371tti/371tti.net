use kurosabi::utils::url_decode_fast;

use crate::{
    file::Content,
    markdown::PageMeta,
    web::{context::SiteContextShared, ogp::img::OgpData},
};

pub struct ThumbnailAPI;

impl ThumbnailAPI {
    pub async fn render_png(s_ctx: &SiteContextShared, path: &[&str]) -> Option<Vec<u8>> {
        let display_path = Self::display_path(path);

        let ogp = match s_ctx.file_service.get_content(path).await {
            Some(Content::HtmlHtml { meta, .. }) | Some(Content::MdHtml { meta, .. }) => {
                Self::from_page_meta(&display_path, &meta)
            }
            Some(Content::DirListing(_)) => {
                Self::from_directory_with_index_meta(s_ctx, path, &display_path).await
            }
            Some(Content::BinaryContent) => Self::from_binary(&display_path),
            None => return None,
        };

        ogp.render_png()
    }

    async fn from_directory_with_index_meta(
        s_ctx: &SiteContextShared,
        path: &[&str],
        display_path: &str,
    ) -> OgpData {
        for file_name in ["index.html", "index.md"] {
            let path_with_index = Self::path_with_child(path, file_name);
            match s_ctx.file_service.get_content(&path_with_index).await {
                Some(Content::HtmlHtml { meta, .. }) | Some(Content::MdHtml { meta, .. }) => {
                    return Self::from_page_meta(display_path, &meta);
                }
                _ => {}
            }
        }

        Self::from_directory(display_path)
    }

    fn from_page_meta(display_path: &str, meta: &PageMeta) -> OgpData {
        let title = meta.title();

        let mut description = meta.description();
        if description.is_empty() {
            description = format!("Preview for {}", title);
        }

        let mut authors = meta.authors.clone().unwrap_or_default();
        if authors.is_empty() {
            authors.push("Unknown".to_string());
        }

        let tags = meta.tags();

        OgpData::new(display_path.to_string(), title, description, authors, tags)
    }

    fn from_directory(display_path: &str) -> OgpData {
        let title = if display_path == "/" {
            "Root Directory".to_string()
        } else {
            display_path
                .trim_start_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("Directory")
                .to_string()
        };

        OgpData::new(
            display_path.to_string(),
            title,
            "Directory listing".to_string(),
            vec!["system".to_string()],
            vec!["directory".to_string()],
        )
    }

    fn from_binary(display_path: &str) -> OgpData {
        let title = display_path
            .trim_start_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("Binary")
            .to_string();

        OgpData::new(
            display_path.to_string(),
            title,
            "Binary file".to_string(),
            vec!["system".to_string()],
            vec!["file".to_string()],
        )
    }

    fn path_with_child<'a>(path: &[&'a str], child: &'a str) -> Vec<&'a str> {
        let mut out = if path == [""] {
            Vec::new()
        } else {
            path.to_vec()
        };
        out.push(child);
        out
    }

    fn display_path(path: &[&str]) -> String {
        let decoded_segments: Vec<String> = path
            .iter()
            .filter(|seg| !seg.is_empty())
            .map(|seg| url_decode_fast(seg).to_string())
            .collect();

        if decoded_segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", decoded_segments.join("/"))
        }
    }
}
