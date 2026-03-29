use std::sync::Arc;

use kurosabi::{
    connection::file::DirEntryInfo,
    utils::{url_decode_fast, url_encode},
};

use crate::{
    config::Config,
    file::Content,
    markdown::PageMeta,
    web::{context::SiteContextShared, templates::TemplateService},
};

#[derive(Clone)]
pub struct DocsRouter;

impl DocsRouter {
    pub fn new(_config: Arc<Config>) -> Self {
        Self
    }

    pub async fn route(
        &self,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> std::io::Result<Option<String>> {
        match s_ctx.file_service.get_content(path).await {
            Some(Content::HtmlHtml { html, meta }) => {
                Ok(Some(TemplateService::render_common_html(html, meta, s_ctx)))
            }
            Some(Content::MdHtml { html, meta }) => {
                Ok(Some(TemplateService::render_common_page(html, meta, s_ctx)))
            }
            Some(Content::DirListing(dir)) => {
                let html = self.render_dir(dir, path, s_ctx).await?;
                Ok(Some(html))
            }
            Some(Content::BinaryContent) => Ok(None),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )),
        }
    }

    async fn render_dir(
        &self,
        dir: Vec<DirEntryInfo>,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> std::io::Result<String> {
        if let Some(rendered_html) = self.try_render_directory_index_html(path, s_ctx).await {
            return Ok(rendered_html);
        }

        let (dirs, files) = Self::collect_visible_entries(&dir);
        let path_segments = Self::decoded_path_segments(path);
        let encoded_path = Self::encoded_path_segments(&path_segments);
        let listing_html =
            Self::build_directory_listing_html(&path_segments, &encoded_path, &dirs, &files);

        let (meta, body) = match self.try_load_directory_index_md(path, s_ctx).await {
            Some((meta, index_html)) => {
                let mut body = String::with_capacity(index_html.len() + listing_html.len() + 8);
                body.push_str(&index_html);
                body.push_str("\n<hr>\n");
                body.push_str(&listing_html);
                (meta, body)
            }
            None => (Self::default_directory_meta(&path_segments), listing_html),
        };

        Ok(TemplateService::render_common_page(body, meta, s_ctx))
    }

    async fn try_render_directory_index_html(
        &self,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> Option<String> {
        let path_with_index = Self::path_with_child(path, "index.html");
        match s_ctx.file_service.get_content(&path_with_index).await? {
            Content::HtmlHtml { html, meta } | Content::MdHtml { html, meta } => {
                Some(TemplateService::render_common_html(html, meta, s_ctx))
            }
            _ => None,
        }
    }

    async fn try_load_directory_index_md(
        &self,
        path: &[&str],
        s_ctx: &SiteContextShared,
    ) -> Option<(PageMeta, String)> {
        let path_with_index = Self::path_with_child(path, "index.md");
        match s_ctx.file_service.get_content(&path_with_index).await? {
            Content::MdHtml { html, meta } | Content::HtmlHtml { html, meta } => Some((meta, html)),
            _ => None,
        }
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

    fn collect_visible_entries<'a>(entries: &'a [DirEntryInfo]) -> (Vec<&'a str>, Vec<&'a str>) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries {
            if entry.kind.is_dir() {
                if let Some(name) = entry.path.file_name().and_then(|n| n.to_str())
                    && !name.starts_with('.')
                {
                    dirs.push(name);
                }
                continue;
            }

            if entry.kind.is_file()
                && let Some(name) = entry.path.file_name().and_then(|n| n.to_str())
                && !name.starts_with('.')
                && name != "index.md"
            {
                files.push(name);
            }
        }

        dirs.sort_unstable();
        files.sort_unstable();
        (dirs, files)
    }

    fn decoded_path_segments(path: &[&str]) -> Vec<String> {
        path.iter()
            .filter(|p| !p.is_empty())
            .map(|s| url_decode_fast(s).to_string())
            .collect()
    }

    fn encoded_path_segments(path_segments: &[String]) -> String {
        let mut out = String::new();
        for (i, segment) in path_segments.iter().enumerate() {
            if i > 0 {
                out.push('/');
            }
            out.push_str(&url_encode(segment));
        }
        out
    }

    fn build_directory_listing_html(
        path_segments: &[String],
        encoded_path: &str,
        dirs: &[&str],
        files: &[&str],
    ) -> String {
        let mut html = String::with_capacity(256 + dirs.len() * 64 + files.len() * 64);
        html.push_str("<section class=\"directory-list\">");
        Self::append_breadcrumb_html(&mut html, path_segments);
        Self::append_link_list_html(&mut html, "Directories", encoded_path, dirs, true);
        Self::append_link_list_html(&mut html, "Files", encoded_path, files, false);
        html.push_str("</section>");
        html
    }

    fn append_breadcrumb_html(out: &mut String, path_segments: &[String]) {
        out.push_str("<h2>Index of <a href=\"/\">root</a>");
        if path_segments.is_empty() {
            out.push_str("</h2>");
            return;
        }

        out.push('/');
        let mut encoded_prefix = String::new();
        for (i, segment) in path_segments.iter().enumerate() {
            if !encoded_prefix.is_empty() {
                encoded_prefix.push('/');
            }
            encoded_prefix.push_str(&url_encode(segment));

            if i + 1 == path_segments.len() {
                Self::push_escaped_html(out, segment);
            } else {
                out.push_str("<a href=\"/");
                out.push_str(&encoded_prefix);
                out.push_str("\">");
                Self::push_escaped_html(out, segment);
                out.push_str("</a>/");
            }
        }
        out.push_str("</h2>");
    }

    fn append_link_list_html(
        out: &mut String,
        title: &str,
        encoded_path: &str,
        names: &[&str],
        trailing_slash: bool,
    ) {
        if names.is_empty() {
            return;
        }

        out.push_str("<h3>");
        Self::push_escaped_html(out, title);
        out.push_str("</h3><ul>");

        for name in names {
            out.push_str("<li><a href=\"/");
            if !encoded_path.is_empty() {
                out.push_str(encoded_path);
                out.push('/');
            }
            out.push_str(&url_encode(name));
            if trailing_slash {
                out.push('/');
            }
            out.push_str("\">");
            Self::push_escaped_html(out, name);
            out.push_str("</a></li>");
        }

        out.push_str("</ul>");
    }

    fn default_directory_meta(path_segments: &[String]) -> PageMeta {
        PageMeta {
            title: Some(
                path_segments
                    .last()
                    .map_or("Root", String::as_str)
                    .to_string(),
            ),
            authors: Some(vec!["system".to_string()]),
            is_complete: true,
            ..Default::default()
        }
    }

    fn push_escaped_html(out: &mut String, raw: &str) {
        for ch in raw.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                _ => out.push(ch),
            }
        }
    }
}
