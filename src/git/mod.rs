use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use gix::{Repository, bstr::ByteSlice};
use log::{error, info};

use crate::config::Config;

pub struct GitService {
    repo: Repository,
    config: Arc<Config>,
}

impl GitService {
    pub async fn load_or_clone_async(config: Arc<Config>) -> Result<(), GitServiceError> {
        tokio::task::spawn_blocking(move || Self::load_or_clone(config).map(|_| ()))
            .await
            .map_err(GitServiceError::FailedJoinTask)?
    }

    pub async fn update_async(config: Arc<Config>) -> Result<String, GitServiceError> {
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
        if repo_path.exists() {
            info!("Repository already exists at {:?}, opening...", repo_path);
            let repo = gix::open(repo_path).map_err(GitServiceError::FailedOpenRepository)?; // 既存リポジトリを開く
            let service = Self { repo, config };
            service.check_repo_remote_conf(service.repo.clone())?;
            Ok(service)
        } else {
            info!("Repository does not exist at {:?}, cloning...", repo_path);
            let auth_able_url = config.git_config.auth_able_url();
            let refname = config.git_config.refname();
            let repo = Self::clone_repo(&repo_path, &auth_able_url, &refname)?;
            Ok(Self { repo, config })
        }
    }

    pub fn update(&self) -> Result<String, GitServiceError> {
        self.pull_ff_only(&self.config.git_config.remote_branch)
            .and_then(|_| self.get_commit_hash())
    }

    pub fn get_commit_hash(&self) -> Result<String, GitServiceError> {
        let mut head = self.repo.head().map_err(GitServiceError::FailedReadHead)?;
        let commit = head
            .peel_to_commit()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;
        let hash = commit.id;
        Ok(hash.to_string())
    }

    /// ? これは違う configに設定追加したので remote enableならdirがなければclone ちがえばswap clone ブランチがちがえば pull みたいな感じでいいかも
    fn check_repo_remote_conf(&self, repo: Repository) -> Result<Repository, GitServiceError> {
        let url = &self.config.git_config.remote_url;
        let ref_name = &self.config.git_config.refname();

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

        let repo_remote = config.string("branch.main.remote").ok_or_else(|| {
            GitServiceError::NotFoundRemoteConfig {
                key: "branch.main.remote".to_string(),
            }
        })?;

        let repo_merge = config.string("branch.main.merge").ok_or_else(|| {
            GitServiceError::NotFoundRemoteConfig {
                key: "branch.main.merge".to_string(),
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
            .with_url_without_url_rewrite(self.config.git_config.auth_able_url().as_bytes().as_bstr())
            .map_err(|err| {
                GitServiceError::FailedOperation(format!("Failed to set remote URL: {}", err))
            })?;

        let should_interrupt = AtomicBool::new(false);
        let mut progress = gix::progress::Discard;

        let connection = remote.connect(gix::remote::Direction::Fetch).map_err(|e| {
            error!("remote connect error: {:?}", e);
            GitServiceError::FailedRemoteConnect(e)
        })?;

        let _outcome = connection
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
            .into_iter(std::iter::empty())
            .map_err(GitServiceError::FailedStatusIter)?;

        if iter.next().is_some() {
            return Err(GitServiceError::WorktreeDirty);
        }
        Ok(())
    }

    pub fn pull_ff_only(&self, branch: &str) -> Result<(), GitServiceError> {
        // 1) fetch (object store + refs/remotes/origin/* 更新)
        self.fetch_origin()?;

        // 2) ローカル変更があれば止める（競合回避）
        self.ensure_clean_worktree()?;

        // 3) local / remote OID を解決
        let (local_ref, local_oid) = self.resolve_branch_head(branch)?;
        let remote_oid = self.resolve_origin_head(branch)?;

        // 既に最新なら何もしない
        if local_oid == remote_oid {
            return Ok(());
        }

        // 4) fast-forward 可能か（local が remote の祖先か）
        if !Self::is_ancestor_gix_only(&self.repo, local_oid, remote_oid)? {
            return Err(GitServiceError::NonFastForward);
        }

        // 5) refs/heads/<branch> を remote に進める（古い値一致を要求して安全に）
        self.repo
            .reference(
                local_ref,
                remote_oid,
                gix::refs::transaction::PreviousValue::MustExistAndMatch(local_oid.into()),
                "fast-forward (ff-only) from origin",
            )
            .map_err(GitServiceError::FailedUpdateBranchRef)?;

        // 6) worktree を remote_oid の tree に同期（checkout）
        let interrupt = AtomicBool::new(false);
        Self::checkout_to_commit_gix_only(&self.repo, remote_oid, &interrupt)?;

        Ok(())
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
        // 典型: FF 判定なので remote 側から遡るのが自然
        let mut stack = vec![descendant];

        while let Some(oid) = stack.pop() {
            if oid == ancestor {
                return Ok(true);
            }

            let commit = repo
                .find_object(oid)
                .map_err(GitServiceError::FailedFindObject)?
                .try_into_commit()
                .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

            for parent in commit.parent_ids() {
                // parent_ids() は ObjectId を返す
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

        let mut index = gix::index::State::from_tree(&tree.id, &repo.objects, Default::default())
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        // checkout は Send を要求する都合で objects を Arc-backed にするのが無難
        let objects = repo
            .objects
            .clone()
            .into_arc()
            .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        let workdir = repo
            .workdir()
            .ok_or(GitServiceError::FailedOperation("No workdir".to_string()))?
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
        .map_err(|err| GitServiceError::FailedOperation(err.to_string()))?;

        Ok(())
    }
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
                write!(f, "Remote configuration not found for key: {}", key)
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
            GitServiceError::FailedFetch(err) => write!(f, "Failed to fetch repository: {:?}", err),
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
            GitServiceError::WorktreeDirty => write!(f, "Worktree has uncommitted changes"),
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
            GitServiceError::NonFastForward => write!(f, "Fast-forward is not possible"),
            GitServiceError::FailedUpdateBranchRef(err) => {
                write!(f, "Failed to update local branch ref: {:?}", err)
            }
            GitServiceError::FailedFindObject(err) => write!(f, "Failed to find object: {:?}", err),
            GitServiceError::FailedJoinTask(err) => {
                write!(f, "Failed to join blocking task: {:?}", err)
            }
            GitServiceError::FailedOperation(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::fmt::Debug for GitServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitServiceError::MismatchRemoteUrl => write!(f, "GitServiceError::MismatchRemoteUrl"),
            GitServiceError::MismatchRemoteBranch => {
                write!(f, "GitServiceError::MismatchRemoteBranch")
            }
            GitServiceError::MismatchRemoteName => write!(f, "GitServiceError::MismatchRemoteName"),
            GitServiceError::NotFoundRemoteConfig { key } => write!(
                f,
                "GitServiceError::NotFoundRemoteConfig {{ key: {} }}",
                key
            ),
            GitServiceError::FailedOpenRepository(err) => {
                write!(f, "GitServiceError::FailedOpenRepository({:?})", err)
            }
            GitServiceError::FailedCloneConfig(err) => {
                write!(f, "GitServiceError::FailedCloneConfig({:?})", err)
            }
            GitServiceError::FailedCloneBranchConfig(err) => {
                write!(f, "GitServiceError::FailedCloneBranchConfig({:?})", err)
            }
            GitServiceError::FailedFetch(err) => {
                write!(f, "GitServiceError::FailedFetch({:?})", err)
            }
            GitServiceError::FailedCheckout(err) => {
                write!(f, "GitServiceError::FailedCheckout({:?})", err)
            }
            GitServiceError::FailedFetchOrigin(err) => {
                write!(f, "GitServiceError::FailedFetchOrigin({:?})", err)
            }
            GitServiceError::FailedRemoteConnect(err) => {
                write!(f, "GitServiceError::FailedRemoteConnect({:?})", err)
            }
            GitServiceError::FailedRemotePrepareFetch(err) => {
                write!(f, "GitServiceError::FailedRemotePrepareFetch({:?})", err)
            }
            GitServiceError::FailedRemoteReceive(err) => {
                write!(f, "GitServiceError::FailedRemoteReceive({:?})", err)
            }
            GitServiceError::FailedStatus(err) => {
                write!(f, "GitServiceError::FailedStatus({:?})", err)
            }
            GitServiceError::FailedStatusIter(err) => {
                write!(f, "GitServiceError::FailedStatusIter({:?})", err)
            }
            GitServiceError::WorktreeDirty => write!(f, "GitServiceError::WorktreeDirty"),
            GitServiceError::FailedReadHead(err) => {
                write!(f, "GitServiceError::FailedReadHead({:?})", err)
            }
            GitServiceError::FailedFindLocalRef(err) => {
                write!(f, "GitServiceError::FailedFindLocalRef({:?})", err)
            }
            GitServiceError::FailedPeelLocalToId(err) => {
                write!(f, "GitServiceError::FailedPeelLocalToId({:?})", err)
            }
            GitServiceError::FailedFindTrackingRef(err) => {
                write!(f, "GitServiceError::FailedFindTrackingRef({:?})", err)
            }
            GitServiceError::FailedPeelTrackingToId(err) => {
                write!(f, "GitServiceError::FailedPeelTrackingToId({:?})", err)
            }
            GitServiceError::NonFastForward => write!(f, "GitServiceError::NonFastForward"),
            GitServiceError::FailedUpdateBranchRef(err) => {
                write!(f, "GitServiceError::FailedUpdateBranchRef({:?})", err)
            }
            GitServiceError::FailedFindObject(err) => {
                write!(f, "GitServiceError::FailedFindObject({:?})", err)
            }
            GitServiceError::FailedJoinTask(err) => {
                write!(f, "GitServiceError::FailedJoinTask({:?})", err)
            }
            GitServiceError::FailedOperation(msg) => {
                write!(f, "GitServiceError::FailedOperation({})", msg)
            }
        }
    }
}

impl Error for GitServiceError {}
