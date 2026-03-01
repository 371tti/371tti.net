use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
/// base_dir(name) と content_repo_url を受け取る
/// base_dir.tmp を作成して urlからzipをダウンロードしてtar crate + zstd crate で解凍して展開する(cli依存しない)
/// base_dirをbase_dir.oldにリネームしてbase_dir.tmpをbase_dirにリネームする
/// 適応したcommitのhashを記録しておいて次回からはそれと比較して更新が必要か判断する
///
/// use std::fmt;
use std::{fmt, fs};

use log::{debug, info, warn};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub updated: bool,
    pub previous_hash: Option<String>,
    pub current_hash: String,
    pub base_dir: PathBuf,
    pub archive_url: String,
}

#[derive(Debug)]
pub enum UpdateError {
    Io(io::Error),
    Http(reqwest::Error),
    HttpStatus { url: String, status: StatusCode },
    InvalidBaseDir(String),
    InvalidArchivePath(PathBuf),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Http(e) => write!(f, "http error: {e}"),
            Self::HttpStatus { url, status } => write!(f, "http status error: {status} ({url})"),
            Self::InvalidBaseDir(s) => write!(f, "invalid base_dir: {s}"),
            Self::InvalidArchivePath(p) => write!(f, "invalid archive path: {}", p.display()),
        }
    }
}

impl std::error::Error for UpdateError {}

impl From<io::Error> for UpdateError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for UpdateError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

pub struct UpdateService;

#[derive(Debug, Deserialize)]
struct GitHubCommitResponse {
    sha: String,
}

impl UpdateService {
    fn trim_url(url: &str) -> &str {
        url.trim_end_matches('/')
    }

    fn parse_github_owner_repo(content_repo_url: &str) -> Option<(String, String)> {
        let url = Self::trim_url(content_repo_url);
        let marker = "github.com/";
        let idx = url.find(marker)?;
        let rest = &url[(idx + marker.len())..];
        let mut parts = rest.split('/').filter(|s| !s.is_empty());
        let owner = parts.next()?;
        let mut repo = parts.next()?.to_string();
        if let Some(stripped) = repo.strip_suffix(".git") {
            repo = stripped.to_string();
        }
        Some((owner.to_string(), repo))
    }

    fn archive_url(content_repo_url: &str, hash: &str) -> String {
        format!(
            "{}/archive/{}.tar.zst",
            Self::trim_url(content_repo_url),
            hash
        )
    }

    fn sibling_with_suffix(base_dir: &Path, suffix: &str) -> Result<PathBuf, UpdateError> {
        let parent = base_dir.parent().unwrap_or_else(|| Path::new("."));
        let name = base_dir
            .file_name()
            .ok_or_else(|| UpdateError::InvalidBaseDir(base_dir.display().to_string()))?
            .to_string_lossy();
        Ok(parent.join(format!("{name}.{suffix}")))
    }

    fn fetch_latest_hash_from_github_api(
        client: &Client,
        content_repo_url: &str,
        branch: &str,
    ) -> Result<String, UpdateError> {
        let (owner, repo) = Self::parse_github_owner_repo(content_repo_url).ok_or_else(|| {
            UpdateError::InvalidBaseDir(format!(
                "content_repo_url is not a GitHub repository URL: {}",
                content_repo_url
            ))
        })?;

        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/commits/{branch}",
            owner = owner,
            repo = repo,
            branch = branch
        );

        debug!("fetch latest commit from api: {}", url);
        let resp = client
            .get(&url)
            .header(USER_AGENT, "wk-371tti-net-updater")
            .header(ACCEPT, "application/vnd.github+json")
            .send()?;

        if !resp.status().is_success() {
            warn!("latest commit api failed: {} status={}", url, resp.status());
            return Err(UpdateError::HttpStatus {
                url,
                status: resp.status(),
            });
        }

        let body = resp.text()?;
        let parsed: GitHubCommitResponse = serde_json::from_str(&body).map_err(|e| {
            UpdateError::InvalidBaseDir(format!("failed to parse github api response: {}", e))
        })?;

        Ok(parsed.sha)
    }

    fn move_git_metadata_if_exists(from_base: &Path, to_base: &Path) -> Result<(), UpdateError> {
        let from_git = from_base.join(".git");
        if !from_git.exists() {
            debug!("no .git metadata found in {}", from_base.display());
            return Ok(());
        }

        let to_git = to_base.join(".git");
        if to_git.exists() {
            if to_git.is_dir() {
                fs::remove_dir_all(&to_git)?;
            } else {
                fs::remove_file(&to_git)?;
            }
        }

        // .git がディレクトリでもファイル（submodule の gitfile）でも rename で移動可能
        fs::rename(from_git, to_git)?;
        info!(
            "preserved .git metadata: {} -> {}",
            from_base.display(),
            to_base.display()
        );
        Ok(())
    }

    fn extract_tar_zst_strip_first<R: Read>(reader: R, dest: &Path) -> Result<(), UpdateError> {
        let decoder = zstd::stream::read::Decoder::new(reader)?;
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let in_path = entry.path()?.to_path_buf();

            // 先頭ディレクトリを除去（GitHubアーカイブ等想定）
            let mut comps = in_path.components();
            let _ = comps.next();
            let rel = comps.as_path().to_path_buf();

            if rel.as_os_str().is_empty() {
                continue;
            }

            for c in rel.components() {
                if matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                ) {
                    return Err(UpdateError::InvalidArchivePath(rel));
                }
            }

            let out = dest.join(&rel);
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(&out)?;
        }

        Ok(())
    }

    /// base_dir と content_repo_url を使って更新する
    /// - branch の最新コミットを API から取得する
    /// - current_hash は前回適用済みハッシュ
    /// - archive/{hash}.tar.zst を使用
    /// - hash が同じなら更新しない
    pub fn update_content(
        base_dir: &Path,
        content_repo_url: &str,
        branch: &str,
        current_hash: Option<&str>,
    ) -> Result<UpdateResult, UpdateError> {
        info!(
            "content update start: base_dir={} repo={} branch={}",
            base_dir.display(),
            content_repo_url,
            branch
        );
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        let previous_hash = current_hash.map(ToOwned::to_owned);
        let latest_hash =
            Self::fetch_latest_hash_from_github_api(&client, content_repo_url, branch)?;
        debug!(
            "hash check: previous={:?} latest={}",
            previous_hash, latest_hash
        );

        if previous_hash.as_deref() == Some(latest_hash.as_str()) {
            info!("content update skipped: hash unchanged ({})", latest_hash);
            return Ok(UpdateResult {
                updated: false,
                previous_hash,
                current_hash: latest_hash.clone(),
                base_dir: base_dir.to_path_buf(),
                archive_url: Self::archive_url(content_repo_url, &latest_hash),
            });
        }

        let tmp_dir = Self::sibling_with_suffix(base_dir, "tmp")?;
        let old_dir = Self::sibling_with_suffix(base_dir, "old")?;

        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        fs::create_dir_all(&tmp_dir)?;

        let arc_url = Self::archive_url(content_repo_url, &latest_hash);
        info!("downloading archive: {}", arc_url);
        let mut resp = client.get(&arc_url).send()?;
        if !resp.status().is_success() {
            warn!(
                "archive download failed: {} status={}",
                arc_url,
                resp.status()
            );
            return Err(UpdateError::HttpStatus {
                url: arc_url,
                status: resp.status(),
            });
        }

        info!("extracting archive into {}", tmp_dir.display());
        Self::extract_tar_zst_strip_first(&mut resp, &tmp_dir)?;

        if base_dir.exists() {
            Self::move_git_metadata_if_exists(base_dir, &tmp_dir)?;
        }

        if old_dir.exists() {
            fs::remove_dir_all(&old_dir)?;
        }
        if base_dir.exists() {
            fs::rename(base_dir, &old_dir)?;
        }

        if let Some(parent) = base_dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&tmp_dir, base_dir)?;
        info!(
            "content update applied: base_dir={} hash={}",
            base_dir.display(),
            latest_hash
        );

        Ok(UpdateResult {
            updated: true,
            previous_hash,
            current_hash: latest_hash.clone(),
            base_dir: base_dir.to_path_buf(),
            archive_url: Self::archive_url(content_repo_url, &latest_hash),
        })
    }
}
