use kurosabi::{connection::file::DirEntryInfo, utils::url_decode_fast};
use serde::Serialize;

use crate::{file::Content, markdown::PageMeta, web::context::SiteContextShared};

pub struct FsAPI;

pub enum FsLsError {
    NotFound,
    NotDirectory,
}

pub enum FsStatError {
    NotFound,
}

#[derive(Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub kind: &'static str,
}

#[derive(Serialize)]
pub struct FsLsResponse {
    pub path: String,
    pub dir_count: usize,
    pub file_count: usize,
    pub dirs: Vec<FsEntry>,
    pub files: Vec<FsEntry>,
}

#[derive(Serialize)]
pub struct FsStatResponse {
    pub path: String,
    pub kind: &'static str,
    pub title: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
    pub is_complete: Option<bool>,
    pub dir_count: Option<usize>,
    pub file_count: Option<usize>,
}

impl FsAPI {
    pub async fn ls(s_ctx: &SiteContextShared, path: &[&str]) -> Result<FsLsResponse, FsLsError> {
        match s_ctx.file_service.get_content(path).await {
            Some(Content::DirListing(entries)) => {
                let (dirs, files) = Self::collect_visible_entries(path, &entries);
                Ok(FsLsResponse {
                    path: Self::display_path(path),
                    dir_count: dirs.len(),
                    file_count: files.len(),
                    dirs,
                    files,
                })
            }
            Some(_) => Err(FsLsError::NotDirectory),
            None => Err(FsLsError::NotFound),
        }
    }

    pub async fn stat(
        s_ctx: &SiteContextShared,
        path: &[&str],
    ) -> Result<FsStatResponse, FsStatError> {
        let display_path = Self::display_path(path);

        match s_ctx.file_service.get_content(path).await {
            Some(Content::DirListing(entries)) => {
                let (dir_count, file_count) = Self::count_visible_entries(&entries);
                Ok(FsStatResponse {
                    path: display_path,
                    kind: "directory",
                    title: None,
                    description: None,
                    authors: Vec::new(),
                    tags: Vec::new(),
                    is_complete: None,
                    dir_count: Some(dir_count),
                    file_count: Some(file_count),
                })
            }
            Some(Content::MdHtml { meta, .. }) => {
                Ok(Self::stat_from_meta(display_path, "markdown", &meta))
            }
            Some(Content::HtmlHtml { meta, .. }) => {
                Ok(Self::stat_from_meta(display_path, "html", &meta))
            }
            Some(Content::BinaryContent) => Ok(FsStatResponse {
                path: display_path,
                kind: "binary",
                title: None,
                description: None,
                authors: Vec::new(),
                tags: Vec::new(),
                is_complete: None,
                dir_count: None,
                file_count: None,
            }),
            None => Err(FsStatError::NotFound),
        }
    }

    fn stat_from_meta(path: String, kind: &'static str, meta: &PageMeta) -> FsStatResponse {
        FsStatResponse {
            path,
            kind,
            title: meta.title.clone(),
            description: meta.description.clone(),
            authors: meta.authors.clone().unwrap_or_default(),
            tags: meta.tags(),
            is_complete: Some(meta.is_complete),
            dir_count: None,
            file_count: None,
        }
    }

    fn collect_visible_entries(
        path: &[&str],
        entries: &[DirEntryInfo],
    ) -> (Vec<FsEntry>, Vec<FsEntry>) {
        let base_path = Self::display_path(path);
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries {
            let Some(name) = entry.path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };

            if name.starts_with('.') {
                continue;
            }

            let entry_path = Self::join_display_path(&base_path, name);

            if entry.kind.is_dir() {
                dirs.push(FsEntry {
                    name: name.to_string(),
                    path: entry_path,
                    kind: "dir",
                });
            } else if entry.kind.is_file() {
                files.push(FsEntry {
                    name: name.to_string(),
                    path: entry_path,
                    kind: "file",
                });
            }
        }

        dirs.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        files.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        (dirs, files)
    }

    fn count_visible_entries(entries: &[DirEntryInfo]) -> (usize, usize) {
        let mut dirs = 0;
        let mut files = 0;

        for entry in entries {
            let Some(name) = entry.path.file_name().and_then(|v| v.to_str()) else {
                continue;
            };

            if name.starts_with('.') {
                continue;
            }

            if entry.kind.is_dir() {
                dirs += 1;
            } else if entry.kind.is_file() {
                files += 1;
            }
        }

        (dirs, files)
    }

    fn join_display_path(base_path: &str, name: &str) -> String {
        if base_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", base_path, name)
        }
    }

    fn display_path(path: &[&str]) -> String {
        let segments: Vec<String> = path
            .iter()
            .filter(|seg| !seg.is_empty())
            .map(|seg| url_decode_fast(seg).to_string())
            .collect();

        if segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", segments.join("/"))
        }
    }
}
