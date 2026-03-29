use std::{fs::FileType, path::{Path, PathBuf}, sync::Arc};

use kurosabi::connection::file::{DirEntryInfo, FileContentBuilder, FileContentBuilderReady};
use tokio::io::AsyncReadExt;

use crate::{
    config::Config, markdown::PageMeta, utils::string_on_memory_size_hint,
    web::TemplateService,
};

pub type PathStr = String;

pub struct FileService {
    pub config: Arc<Config>,
    pub rendered_cache: moka::future::Cache<PathStr, Content>,
}

#[derive(Clone)]
pub enum Content {
    HtmlHtml { html: String, meta: PageMeta },
    MdHtml { html: String, meta: PageMeta },
    DirListing(Vec<DirEntryInfo>),
    BinaryContent,
}

impl Content {
    pub fn on_memory_size_hint(&self) -> usize {
        match self {
            Content::HtmlHtml { html, meta } => {
                string_on_memory_size_hint(html)
                    + meta.on_memory_size_hint()
                    + std::mem::size_of::<Content>()
            }
            Content::MdHtml { html, meta } => {
                string_on_memory_size_hint(html)
                    + meta.on_memory_size_hint()
                    + std::mem::size_of::<Content>()
            }
            Content::DirListing(entries) => {
                entries
                    .iter()
                    .map(|e| {
                        e.path.capacity()
                            + std::mem::size_of::<PathBuf>()
                            + std::mem::size_of::<FileType>()
                            + std::mem::size_of::<DirEntryInfo>()
                    })
                    .sum::<usize>()
                    + std::mem::size_of::<Content>()
            }
            Content::BinaryContent => std::mem::size_of::<Content>(),
        }
    }
}

impl FileService {
    pub fn new(config: Arc<Config>) -> Self {
        let rendered_cache = moka::future::Cache::<String, Content>::builder()
            .max_capacity(config.cache_config.max_memory_bytes)
            .weigher(|key, value| {
                (string_on_memory_size_hint(key) + value.on_memory_size_hint()) as u32
            })
            .build();
        Self {
            config,
            rendered_cache,
        }
    }

    pub async fn get_from_cache(&self, path: &str) -> Option<Content> {
        self.rendered_cache.get(path).await
    }

    pub async fn put_in_cache(&self, path: String, content: Content) {
        self.rendered_cache.insert(path, content).await;
    }

    pub async fn purge_cache(&self) {
        self.rendered_cache.invalidate_all();
    }
    
    pub async fn purge_cache_by_path(&self, path: &str) {
        self.rendered_cache.invalidate(path).await;
    }

    pub async fn get_content(&self, path: &[&str]) -> Option<Content> {
        let joined_path = path.join("/");
        match self.get_from_cache(&joined_path).await {
            Some(cached) => {
                log::debug!("Cache hit for path: {}", joined_path);
                Some(cached)
            },
            None => {
                match self.builder_ready(path).await {
                    Ok(builder) => {
                        let mut file = builder.build().await.ok()?;
                        match DocKind::classify(&file.mime_type, &file.file_path) {
                            DocKind::Markdown => {
                                let mut buf = String::new();
                                let _bytes = file.file.read_to_string(&mut buf).await.ok()?;
                                let (meta, content_md) = TemplateService::parse_front_matter(buf, path);
                                let html = crate::render::md_to_html_gfm_highlight(&content_md);
                                let content = Content::MdHtml { html, meta };
                                self.put_in_cache(joined_path.clone(), content.clone()).await;
                                Some(content)
                            }
                            DocKind::Html => {
                                let mut buf = String::new();
                                let _bytes = file.file.read_to_string(&mut buf).await.ok()?;
                                let (meta, html) = TemplateService::parse_front_matter(buf, path);
                                let content = Content::HtmlHtml { html, meta };
                                self.put_in_cache(joined_path.clone(), content.clone()).await;
                                Some(content)
                            }
                            DocKind::Other => {
                                // キャッシュには入れない
                                Some(Content::BinaryContent)
                            }
                        }
                    }
                    Err(Some(dir)) => Some(Content::DirListing(dir)),
                    Err(None) => None,
                }
            }
        }
    }

    async fn builder_ready(
        &self,
        path: &[&str],
    ) -> Result<FileContentBuilder<FileContentBuilderReady>, Option<Vec<DirEntryInfo>>> {
        FileContentBuilder::base(&self.config.base_dir)
            .path_url_segs(path)
            .check_file_exists()
            .await
    }
}

pub enum DocKind {
    Markdown,
    Html,
    Other,
}

impl DocKind {
    pub fn classify(mime_type: &str, file_path: &Path) -> Self {
        let file_ext = file_path
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
}