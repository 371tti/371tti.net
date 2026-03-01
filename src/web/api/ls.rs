use kurosabi::connection::file::{DirEntryInfo, FileContentBuilder};
use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub enum LsKind {
    Dir,
    File,
    NotFound,
}

#[derive(Serialize)]
pub struct LsEntry {
    pub name: String,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct LsFile {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
}

#[derive(Serialize)]
pub struct LsResponse {
    pub path: String,
    pub kind: LsKind,
    pub dirs: Vec<LsEntry>,
    pub files: Vec<LsEntry>,
    pub file: Option<LsFile>,
}

impl LsResponse {
    fn not_found(path: &[&str]) -> Self {
        Self {
            path: path.join("/"),
            kind: LsKind::NotFound,
            dirs: Vec::new(),
            files: Vec::new(),
            file: None,
        }
    }

    fn file(path: &[&str], file: LsFile) -> Self {
        Self {
            path: path.join("/"),
            kind: LsKind::File,
            dirs: Vec::new(),
            files: Vec::new(),
            file: Some(file),
        }
    }
}

#[derive(Clone)]
pub struct LsAPI {
    base_dir: String,
}

impl LsAPI {
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub async fn list(&self, path: &[&str]) -> std::io::Result<LsResponse> {
        let builder = match FileContentBuilder::base(&self.base_dir)
            .path_url_segs(path)
            .check_file_exists()
            .await
        {
            Ok(f) => f,
            Err(d) => match d {
                Some(dir) => return Ok(Self::build_dir_response(path, dir)),
                None => return Ok(LsResponse::not_found(path)),
            },
        };

        let file = builder.build().await?;
        let size = file.file.metadata().await?.len();
        let name = path.last().map_or("", |v| v).to_string();
        Ok(LsResponse::file(path, LsFile {
            name,
            size,
            mime_type: file.mime_type,
        }))
    }

    fn build_dir_response(path: &[&str], dir: Vec<DirEntryInfo>) -> LsResponse {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in dir.iter() {
            if entry.kind.is_dir() {
                if let Some(name) = entry.path.file_name().and_then(|n| n.to_str()) {
                    dirs.push(LsEntry {
                        name: name.to_string(),
                        size: None,
                    });
                }
            } else if entry.kind.is_file() {
                if let Some(name) = entry.path.file_name().and_then(|n| n.to_str()) {
                    let size = entry.path.metadata().ok().map(|m| m.len());
                    files.push(LsEntry {
                        name: name.to_string(),
                        size,
                    });
                }
            }
        }
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        LsResponse {
            path: path.join("/"),
            kind: LsKind::Dir,
            dirs,
            files,
            file: None,
        }
    }
}
