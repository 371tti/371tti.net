use std::{
    collections::HashSet,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use gix::{Repository, bstr::ByteSlice, object::tree::EntryMode, objs::tree::EntryKind};
use log::{error, info};

use crate::config::Config;

#[derive(Clone)]
pub struct GitService {
    repo: Repository,
    config: Arc<Config>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileChange {
    Added { path: String },
    Modified { path: String },
    Deleted { path: String },
    Renamed { old_path: String, new_path: String },
}

#[derive(Debug, Clone)]
pub struct PullOutcome {
    pub old_commit: String,
    pub new_commit: String,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateOutcome {
    pub old_commit: String,
    pub new_commit: String,
    pub changed: bool,
    pub file_changes: Vec<FileChange>,
}

impl std::fmt::Display for UpdateOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "UpdateOutcome")?;
        writeln!(f, "old_commit: {},", self.old_commit)?;
        writeln!(f, "new_commit: {},", self.new_commit)?;
        writeln!(f, "changed: {},", self.changed)?;
        writeln!(f, "file_changes: ")?;
        for change in &self.file_changes {
            match change {
                FileChange::Added { path } => {
                    writeln!(f, "  \x1b[32m+\x1b[0m {{ path: {} }},", path)?
                }
                FileChange::Modified { path } => {
                    writeln!(f, "  \x1b[33mM\x1b[0m {{ path: {} }},", path)?
                }
                FileChange::Deleted { path } => {
                    writeln!(f, "  \x1b[31m-\x1b[0m {{ path: {} }},", path)?
                }
                FileChange::Renamed { old_path, new_path } => writeln!(
                    f,
                    "  \x1b[36mR\x1b[0m {{ old_path: {}, new_path: {} }},",
                    old_path, new_path
                )?,
            }
        }
        write!(f, "")
    }
}

impl GitService {
    pub async fn load_or_clone_async(config: Arc<Config>) -> Result<(), GitServiceError> {
        tokio::task::spawn_blocking(move || Self::load_or_clone(config).map(|_| ()))
            .await
            .map_err(GitServiceError::FailedJoinTask)?
    }

    pub async fn update_async(config: Arc<Config>) -> Result<UpdateOutcome, GitServiceError> {
        tokio::task::spawn_blocking(move || {
            let service = Self::load_or_clone(config)?;
            service.update()
        })
        .await
        .map_err(GitServiceError::FailedJoinTask)?
    }

    pub async fn get_commit_hash_async(config: Arc<Config>) -> Result<String, GitServiceError> {
        tokio::task::spawn_blocking(move || {
            let service = Self::load_or_clone(config)?;
            service.get_commit_hash()
        })
        .await
        .map_err(GitServiceError::FailedJoinTask)?
    }

    pub fn load_or_clone(config: Arc<Config>) -> Result<Self, GitServiceError> {
        let repo_path = PathBuf::from(config.base_dir.clone());

        let mut repo = if repo_path.exists() {
            info!("Repository already exists at {:?}, opening...", repo_path);
            gix::open(repo_path).map_err(GitServiceError::FailedOpenRepository)?
        } else {
            info!("Repository does not exist at {:?}, cloning...", repo_path);
            let auth_able_url = config.git_config.auth_able_url();
            let refname = config.git_config.refname();
            Self::clone_repo(&repo_path, &auth_able_url, &refname)?
        };

        // 環境変数やシステム設定にcommitter設定がない場合のフォールバック ないとpullできん
        repo.committer_or_set_generic_fallback().map_err(|err| {
            GitServiceError::FailedOperation(format!("Failed to prepare committer fallback: {err}"))
        })?;

        let service = Self { repo, config };
        service.check_repo_remote_conf(service.repo.clone())?;
        Ok(service)
    }

    pub fn update(&self) -> Result<UpdateOutcome, GitServiceError> {
        let pull = self.pull_ff_only(&self.config.git_config.remote_branch)?;

        if !pull.changed {
            return Ok(UpdateOutcome {
                old_commit: pull.old_commit.clone(),
                new_commit: pull.new_commit.clone(),
                changed: false,
                file_changes: Vec::new(),
            });
        }

        let file_changes =
            self.collect_file_changes_between_commits(&pull.old_commit, &pull.new_commit)?;

        Ok(UpdateOutcome {
            old_commit: pull.old_commit,
            new_commit: pull.new_commit,
            changed: true,
            file_changes,
        })
    }

    pub fn get_commit_hash(&self) -> Result<String, GitServiceError> {
        let mut head = self.repo.head().map_err(GitServiceError::FailedReadHead)?;
        let commit = head
            .peel_to_commit()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;
        Ok(commit.id.to_string())
    }

    fn current_branch_short_name(&self) -> &str {
        &self.config.git_config.remote_branch
    }

    fn check_repo_remote_conf(&self, repo: Repository) -> Result<Repository, GitServiceError> {
        let url = &self.config.git_config.remote_url;
        let branch = self.current_branch_short_name();
        let ref_name = self.config.git_config.refname();

        info!(
            "Checking repository remote configuration for URL: {} and branch ref: {}",
            url, ref_name
        );

        let config = repo.config_snapshot();

        let repo_url = config.string("remote.origin.url").ok_or_else(|| {
            GitServiceError::NotFoundRemoteConfig {
                key: "remote.origin.url".to_string(),
            }
        })?;

        let branch_remote_key = format!("branch.{branch}.remote");
        let branch_merge_key = format!("branch.{branch}.merge");

        let repo_remote = config.string(&branch_remote_key).ok_or_else(|| {
            GitServiceError::NotFoundRemoteConfig {
                key: branch_remote_key.clone(),
            }
        })?;

        let repo_merge = config.string(&branch_merge_key).ok_or_else(|| {
            GitServiceError::NotFoundRemoteConfig {
                key: branch_merge_key.clone(),
            }
        })?;

        if repo_url.as_ref() != url.as_bytes() {
            return Err(GitServiceError::MismatchRemoteUrl);
        }

        if repo_remote.as_ref() != b"origin" {
            return Err(GitServiceError::MismatchRemoteName);
        }

        if repo_merge.as_ref() != ref_name.as_bytes() {
            return Err(GitServiceError::MismatchRemoteBranch);
        }

        Ok(repo)
    }

    fn clone_repo(
        path: &PathBuf,
        auth_able_url: &str,
        refname: &str,
    ) -> Result<Repository, GitServiceError> {
        let mut prep =
            gix::prepare_clone(auth_able_url, path).map_err(GitServiceError::FailedCloneConfig)?;
        prep = prep
            .with_ref_name(Some(refname))
            .map_err(GitServiceError::FailedCloneBranchConfig)?;
        let interrupt = AtomicBool::new(false);
        let (mut pc, _outcome) = prep
            .fetch_then_checkout(gix::progress::Discard, &interrupt)
            .map_err(GitServiceError::FailedFetch)?;
        let (repo, _checkout_outcome) = pc
            .main_worktree(gix::progress::Discard, &interrupt)
            .map_err(GitServiceError::FailedCheckout)?;
        Ok(repo)
    }

    fn fetch_origin(&self) -> Result<(), GitServiceError> {
        let remote = self
            .repo
            .find_remote("origin")
            .map_err(GitServiceError::FailedFetchOrigin)?;

        let remote = remote
            .with_url_without_url_rewrite(
                self.config.git_config.auth_able_url().as_bytes().as_bstr(),
            )
            .map_err(|err| {
                GitServiceError::FailedOperation(format!("Failed to set remote URL: {err}"))
            })?;

        let should_interrupt = AtomicBool::new(false);
        let mut progress = gix::progress::Discard;

        let connection = remote.connect(gix::remote::Direction::Fetch).map_err(|e| {
            error!("remote connect error: {:?}", e);
            GitServiceError::FailedRemoteConnect(e)
        })?;

        connection
            .prepare_fetch(&mut progress, Default::default())
            .map_err(GitServiceError::FailedRemotePrepareFetch)?
            .receive(&mut progress, &should_interrupt)
            .map_err(GitServiceError::FailedRemoteReceive)?;

        Ok(())
    }

    fn ensure_clean_worktree(&self) -> Result<(), GitServiceError> {
        let mut iter = self
            .repo
            .status(gix::progress::Discard)
            .map_err(GitServiceError::FailedStatus)?
            .untracked_files(gix::status::UntrackedFiles::None)
            .into_iter(std::iter::empty())
            .map_err(GitServiceError::FailedStatusIter)?;

        if iter.next().is_some() {
            return Err(GitServiceError::WorktreeDirty);
        }
        Ok(())
    }

    pub fn pull_ff_only(&self, branch: &str) -> Result<PullOutcome, GitServiceError> {
        let old_commit = self.get_commit_hash()?;

        self.fetch_origin()?;
        self.ensure_clean_worktree()?;

        let (local_ref, local_oid) = self.resolve_branch_head(branch)?;
        let remote_oid = self.resolve_origin_head(branch)?;

        if local_oid == remote_oid {
            return Ok(PullOutcome {
                old_commit: old_commit.clone(),
                new_commit: old_commit,
                changed: false,
            });
        }

        if !Self::is_ancestor_gix_only(&self.repo, local_oid, remote_oid)? {
            return Err(GitServiceError::NonFastForward);
        }

        self.repo
            .reference(
                local_ref,
                remote_oid,
                gix::refs::transaction::PreviousValue::MustExistAndMatch(local_oid.into()),
                "fast-forward (ff-only) from origin",
            )
            .map_err(GitServiceError::FailedUpdateBranchRef)?;

        let interrupt = AtomicBool::new(false);
        Self::checkout_to_commit_gix_only(&self.repo, remote_oid, &interrupt)?;

        let new_commit = self.get_commit_hash()?;
        Ok(PullOutcome {
            old_commit,
            new_commit,
            changed: true,
        })
    }

    fn resolve_branch_head(
        &self,
        branch: &str,
    ) -> Result<(String, gix::ObjectId), GitServiceError> {
        let local_ref = format!("refs/heads/{branch}");
        let oid = self
            .repo
            .find_reference(&local_ref)
            .map_err(GitServiceError::FailedFindLocalRef)?
            .peel_to_id()
            .map_err(GitServiceError::FailedPeelLocalToId)?
            .detach();
        Ok((local_ref, oid))
    }

    fn resolve_origin_head(&self, branch: &str) -> Result<gix::ObjectId, GitServiceError> {
        let remote_ref = format!("refs/remotes/origin/{branch}");
        let oid = self
            .repo
            .find_reference(&remote_ref)
            .map_err(GitServiceError::FailedFindTrackingRef)?
            .peel_to_id()
            .map_err(GitServiceError::FailedPeelTrackingToId)?
            .detach();
        Ok(oid)
    }

    fn is_ancestor_gix_only(
        repo: &Repository,
        ancestor: gix::ObjectId,
        descendant: gix::ObjectId,
    ) -> Result<bool, GitServiceError> {
        let mut stack = vec![descendant];
        let mut seen = HashSet::new();

        while let Some(oid) = stack.pop() {
            if !seen.insert(oid) {
                continue;
            }
            if oid == ancestor {
                return Ok(true);
            }

            let commit = repo
                .find_object(oid)
                .map_err(GitServiceError::FailedFindObject)?
                .try_into_commit()
                .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

            for parent in commit.parent_ids() {
                stack.push(parent.detach());
            }
        }

        Ok(false)
    }

    fn checkout_to_commit_gix_only(
        repo: &Repository,
        commit: gix::ObjectId,
        interrupt: &AtomicBool,
    ) -> Result<(), GitServiceError> {
        let commit = repo
            .find_object(commit)
            .map_err(GitServiceError::FailedFindObject)?
            .try_into_commit()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let tree = commit
            .tree()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let mut index = repo.index_from_tree(&tree.id).map_err(|err| {
            GitServiceError::FailedOperation(format!("index_from_tree failed: {err}"))
        })?;

        let objects = repo
            .objects
            .clone()
            .into_arc()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let workdir = repo
            .workdir()
            .ok_or_else(|| GitServiceError::FailedOperation("No workdir".to_string()))?
            .to_path_buf();

        gix::worktree::state::checkout(
            &mut index,
            workdir,
            objects,
            &gix::progress::Discard,
            &gix::progress::Discard,
            interrupt,
            gix::worktree::state::checkout::Options::default(),
        )
        .map_err(|err| GitServiceError::FailedOperation(format!("checkout failed: {err}")))?;

        let index_path = repo.index_path();
        let tmp_path = index_path.with_extension("index.tmp");

        {
            let file = File::create(&tmp_path).map_err(|err| {
                GitServiceError::FailedOperation(format!("create temp index failed: {err}"))
            })?;
            let mut out = BufWriter::new(file);

            index
                .write_to(&mut out, Default::default())
                .map_err(|err| {
                    GitServiceError::FailedOperation(format!("write_to index failed: {err}"))
                })?;

            out.flush().map_err(|err| {
                GitServiceError::FailedOperation(format!("flush temp index failed: {err}"))
            })?;
        }

        fs::rename(&tmp_path, &index_path).map_err(|err| {
            GitServiceError::FailedOperation(format!("replace index failed: {err}"))
        })?;

        Ok(())
    }

    fn collect_file_changes_between_commits(
        &self,
        old_commit: &str,
        new_commit: &str,
    ) -> Result<Vec<FileChange>, GitServiceError> {
        if old_commit == new_commit {
            return Ok(Vec::new());
        }

        let old_commit_id = parse_object_id(old_commit)?;
        let new_commit_id = parse_object_id(new_commit)?;

        let old_commit = self
            .repo
            .find_object(old_commit_id)
            .map_err(GitServiceError::FailedFindObject)?
            .try_into_commit()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let new_commit = self
            .repo
            .find_object(new_commit_id)
            .map_err(GitServiceError::FailedFindObject)?
            .try_into_commit()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let old_tree = old_commit
            .tree()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;
        let new_tree = new_commit
            .tree()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let mut repo = self.repo.clone();

        if let Ok(index) = repo.index_or_empty() {
            let bytes = repo.compute_object_cache_size_for_tree_diffs(&index);
            repo.object_cache_size_if_unset(bytes);
        } else {
            repo.object_cache_size_if_unset(16 * 1024 * 1024);
        }

        let mut opts = gix::diff::Options::default();
        opts.track_path();
        opts.track_rewrites(Some(Default::default()));

        let detached_changes = repo
            .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(opts))
            .map_err(|err| GitServiceError::FailedTreeDiff(err.to_string()))?;

        let mut changes = Vec::with_capacity(detached_changes.len());

        for change in detached_changes {
            use gix::object::tree::diff::ChangeDetached;

            match change {
                ChangeDetached::Addition {
                    location,
                    entry_mode,
                    ..
                } if is_regular_indexable_file_mode(entry_mode) => {
                    changes.push(FileChange::Added {
                        path: bstring_to_string(&location),
                    });
                }
                ChangeDetached::Deletion {
                    location,
                    entry_mode,
                    ..
                } if is_regular_indexable_file_mode(entry_mode) => {
                    changes.push(FileChange::Deleted {
                        path: bstring_to_string(&location),
                    });
                }
                ChangeDetached::Modification {
                    location,
                    entry_mode,
                    ..
                } if is_regular_indexable_file_mode(entry_mode) => {
                    changes.push(FileChange::Modified {
                        path: bstring_to_string(&location),
                    });
                }
                ChangeDetached::Rewrite {
                    source_location,
                    location,
                    source_entry_mode,
                    entry_mode,
                    copy,
                    ..
                } => {
                    let old_path = bstring_to_string(&source_location);
                    let new_path = bstring_to_string(&location);

                    if copy {
                        if is_regular_indexable_file_mode(entry_mode) {
                            changes.push(FileChange::Added { path: new_path });
                        }
                        continue;
                    }

                    let old_ok = is_regular_indexable_file_mode(source_entry_mode);
                    let new_ok = is_regular_indexable_file_mode(entry_mode);

                    match (old_ok, new_ok) {
                        (true, true) => changes.push(FileChange::Renamed { old_path, new_path }),
                        (true, false) => changes.push(FileChange::Deleted { path: old_path }),
                        (false, true) => changes.push(FileChange::Added { path: new_path }),
                        (false, false) => {}
                    }
                }
                _ => {}
            }
        }

        Ok(changes)
    }
}

fn parse_object_id(hex: &str) -> Result<gix::ObjectId, GitServiceError> {
    gix::ObjectId::from_hex(hex.as_bytes()).map_err(|err| {
        GitServiceError::FailedOperation(format!("invalid object id '{hex}': {err}"))
    })
}

fn is_regular_indexable_file_mode(mode: EntryMode) -> bool {
    mode == EntryMode::from(EntryKind::Blob)
        || mode == EntryMode::from(EntryKind::BlobExecutable)
        || mode == EntryMode::from(EntryKind::Link)
}

// utf-8でないパスもあるからバグ要因でもある、 要修正かも

fn bstring_to_string(path: &gix::bstr::BString) -> String {
    String::from_utf8_lossy(path).to_string()
}

pub enum GitServiceError {
    MismatchRemoteUrl,
    MismatchRemoteBranch,
    MismatchRemoteName,
    NotFoundRemoteConfig { key: String },

    FailedOpenRepository(gix::open::Error),
    FailedCloneConfig(gix::clone::Error),
    FailedCloneBranchConfig(gix::refs::name::Error),
    FailedFetch(gix::clone::fetch::Error),
    FailedCheckout(gix::clone::checkout::main_worktree::Error),

    FailedFetchOrigin(gix::remote::find::existing::Error),
    FailedRemoteConnect(gix::remote::connect::Error),
    FailedRemotePrepareFetch(gix::remote::fetch::prepare::Error),
    FailedRemoteReceive(gix::remote::fetch::Error),

    FailedStatus(gix::status::Error),
    FailedStatusIter(gix::status::into_iter::Error),
    WorktreeDirty,

    FailedReadHead(gix::reference::find::existing::Error),
    FailedFindLocalRef(gix::reference::find::existing::Error),
    FailedPeelLocalToId(gix::reference::peel::Error),
    FailedFindTrackingRef(gix::reference::find::existing::Error),
    FailedPeelTrackingToId(gix::reference::peel::Error),
    NonFastForward,
    FailedUpdateBranchRef(gix::reference::edit::Error),
    FailedFindObject(gix::object::find::existing::Error),
    FailedTreeDiff(String),

    FailedJoinTask(tokio::task::JoinError),
    FailedOperation(String),
}

impl std::fmt::Display for GitServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitServiceError::MismatchRemoteUrl => write!(f, "Remote URL mismatch"),
            GitServiceError::MismatchRemoteBranch => write!(f, "Remote branch mismatch"),
            GitServiceError::MismatchRemoteName => write!(f, "Remote name mismatch"),
            GitServiceError::NotFoundRemoteConfig { key } => {
                write!(f, "Remote configuration not found for key: {key}")
            }
            GitServiceError::FailedOpenRepository(err) => {
                write!(f, "Failed to open repository: {:?}", err)
            }
            GitServiceError::FailedCloneConfig(err) => {
                write!(f, "Failed to configure clone: {:?}", err)
            }
            GitServiceError::FailedCloneBranchConfig(err) => {
                write!(f, "Failed to configure clone branch: {:?}", err)
            }
            GitServiceError::FailedFetch(err) => {
                write!(f, "Failed to fetch repository: {:?}", err)
            }
            GitServiceError::FailedCheckout(err) => {
                write!(f, "Failed to checkout worktree: {:?}", err)
            }
            GitServiceError::FailedFetchOrigin(err) => {
                write!(f, "Failed to find origin remote: {:?}", err)
            }
            GitServiceError::FailedRemoteConnect(err) => {
                write!(f, "Failed to connect remote: {:?}", err)
            }
            GitServiceError::FailedRemotePrepareFetch(err) => {
                write!(f, "Failed to prepare remote fetch: {:?}", err)
            }
            GitServiceError::FailedRemoteReceive(err) => {
                write!(f, "Failed to receive remote data: {:?}", err)
            }
            GitServiceError::FailedStatus(err) => {
                write!(f, "Failed to get worktree status: {:?}", err)
            }
            GitServiceError::FailedStatusIter(err) => {
                write!(f, "Failed to iterate worktree status: {:?}", err)
            }
            GitServiceError::WorktreeDirty => {
                write!(f, "Worktree has uncommitted changes")
            }
            GitServiceError::FailedReadHead(err) => write!(f, "Failed to read HEAD: {:?}", err),
            GitServiceError::FailedFindLocalRef(err) => {
                write!(f, "Failed to find local branch ref: {:?}", err)
            }
            GitServiceError::FailedPeelLocalToId(err) => {
                write!(f, "Failed to resolve local branch head: {:?}", err)
            }
            GitServiceError::FailedFindTrackingRef(err) => {
                write!(f, "Failed to find tracking ref: {:?}", err)
            }
            GitServiceError::FailedPeelTrackingToId(err) => {
                write!(f, "Failed to resolve tracking branch head: {:?}", err)
            }
            GitServiceError::NonFastForward => {
                write!(f, "Fast-forward is not possible")
            }
            GitServiceError::FailedUpdateBranchRef(err) => {
                write!(f, "Failed to update local branch ref: {:?}", err)
            }
            GitServiceError::FailedFindObject(err) => {
                write!(f, "Failed to find object: {:?}", err)
            }
            GitServiceError::FailedTreeDiff(err) => {
                write!(f, "Failed to diff trees: {err}")
            }
            GitServiceError::FailedJoinTask(err) => {
                write!(f, "Failed to join blocking task: {:?}", err)
            }
            GitServiceError::FailedOperation(msg) => {
                write!(f, "Operation failed: {msg}")
            }
        }
    }
}

impl std::fmt::Debug for GitServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl Error for GitServiceError {}
