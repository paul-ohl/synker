use git2::Repository;
use tracing::{debug, info, instrument, warn};

use crate::domain::{
    ports,
    services::synchronisation::{SynchronisationError, SynchronisationReport},
};

/// A Git-based implementation of the `Synchronisation` port.
///
/// This synchronizer performs a standard Git workflow: stage changes, commit,
/// pull with rebase, and push to a remote repository.
pub struct GitSynchronizer {
    remote: String,
    branch: String,
    repo_path: String,
    user_email: String,
}

impl GitSynchronizer {
    /// Creates a new instance of `GitSynchronizer`.
    pub fn new(remote: String, branch: String, repo_path: String, user_email: String) -> Self {
        GitSynchronizer {
            remote,
            branch,
            repo_path,
            user_email,
        }
    }

    /// Stages all changes in the repository, equivalent to `git add -A`.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if the index cannot be accessed or updated.
    #[instrument(skip(self, repo))]
    fn add_all(&self, repo: &Repository) -> Result<(), SynchronisationError> {
        debug!("Staging all changes (git add -A)");
        let mut index = repo
            .index()
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;
        index
            .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;
        index
            .write()
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))
    }

    /// Generates a timestamped commit message for the auto-sync.
    fn generate_commit_message(&self, _repo: &Repository) -> Result<String, SynchronisationError> {
        let now = chrono::Local::now();
        Ok(format!("Auto-sync: {}", now.format("%Y-%m-%d %H:%M:%S")))
    }

    /// Creates a new commit with the staged changes.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if the tree cannot be created or the commit fails.
    #[instrument(skip(self, repo), fields(message))]
    fn create_commit(
        &self,
        repo: &Repository,
        message: &str,
    ) -> Result<git2::Oid, SynchronisationError> {
        debug!(message, "Creating auto-sync commit");
        let signature = git2::Signature::now("AutoSync", &self.user_email)?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        let oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent_commit],
            )
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;
        debug!(commit = %oid, "Commit created");
        Ok(oid)
    }

    /// Performs a `git pull --rebase` from the configured remote's branch.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if fetching, rebasing, or committing rebased changes fails.
    /// If a conflict occurs, the rebase is aborted.
    #[instrument(skip(self, repo), fields(remote = %self.remote, branch = %self.branch))]
    fn pull_rebase(&self, repo: &Repository) -> Result<(), SynchronisationError> {
        debug!("Fetching from remote");
        let mut remote = repo.find_remote(&self.remote)?;
        remote.fetch(&[&self.branch], None, None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = fetch_head.peel_to_commit()?;
        let annotated_fetch_commit = repo.find_annotated_commit(fetch_commit.id())?;

        debug!("Starting rebase");
        let mut rebase = repo.rebase(None, Some(&annotated_fetch_commit), None, None)?;
        let signature = git2::Signature::now("AutoSync", &self.user_email)?;

        while let Some(op) = rebase.next() {
            match op {
                Ok(_) => {
                    if repo.index()?.has_conflicts() {
                        warn!("Merge conflict detected during rebase; aborting");
                        rebase.abort()?;
                        return Err(SynchronisationError::FileConflict(
                            "Merge conflict detected during rebase".to_string(),
                        ));
                    }
                    rebase.commit(None, &signature, None)?;
                }
                Err(e) => {
                    warn!(error = %e, "Rebase step failed; aborting");
                    rebase.abort()?;
                    return Err(SynchronisationError::SyncToolError(e.to_string()));
                }
            }
        }
        rebase.finish(None)?;
        debug!("Rebase finished successfully");
        Ok(())
    }

    /// Pushes the local branch to the configured remote.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if the push operation fails.
    #[instrument(skip(self, repo), fields(remote = %self.remote, branch = %self.branch))]
    fn push_to_origin(&self, repo: &Repository) -> Result<(), SynchronisationError> {
        debug!("Pushing to remote");
        let mut remote = repo.find_remote(&self.remote)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", self.branch, self.branch);
        remote.push(&[&refspec], None)?;
        debug!("Push complete");
        Ok(())
    }
}

impl ports::Synchronisation for GitSynchronizer {
    /// Synchronizes the local repository with the configured remote branch.
    ///
    /// The process includes:
    /// 1. Staging all changes (`git add -A`)
    /// 2. Creating an auto-generated commit
    /// 3. Fetching and rebasing from the remote branch
    /// 4. Pushing to the remote branch
    #[instrument(skip(self), fields(remote = %self.remote, branch = %self.branch, repo = %self.repo_path))]
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError> {
        info!("Starting synchronisation");
        let repo = Repository::discover(&self.repo_path)?;

        self.add_all(&repo)?;

        let statuses = repo
            .statuses(None)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;

        if !statuses.is_empty() {
            let commit_message = self.generate_commit_message(&repo)?;
            info!(message = %commit_message, changes = statuses.len(), "Committing staged changes");
            self.create_commit(&repo, &commit_message)?;
        } else {
            debug!("No staged changes; skipping commit");
        }

        self.pull_rebase(&repo)?;
        self.push_to_origin(&repo)?;

        let head = repo.head()?.peel_to_commit()?;
        let commit_sha = head.id().to_string();
        info!(commit = %commit_sha, "Synchronisation complete");

        Ok(SynchronisationReport {
            commit_name: commit_sha,
            last_sync_time: std::time::SystemTime::now(),
            pending_changes: 0,
        })
    }

    /// Provides a report on the current synchronization state.
    #[instrument(skip(self), fields(remote = %self.remote, branch = %self.branch, repo = %self.repo_path))]
    fn get_last_sync(&self) -> Result<SynchronisationReport, SynchronisationError> {
        debug!("Checking last sync state");
        let repo = Repository::discover(&self.repo_path)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;

        // Find the remote reference to determine the last successful sync (push)
        let remote_ref_name = format!("refs/remotes/{}/{}", self.remote, self.branch);
        let remote_ref = repo.find_reference(&remote_ref_name).map_err(|_| {
            debug!("No remote tracking ref found; treating as first-time sync");
            SynchronisationError::FirstTimeSync
        })?;

        let commit = remote_ref
            .peel_to_commit()
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;

        // Calculate commit time
        let commit_time_seconds = commit.time().seconds();
        // If commit time is before UNIX_EPOCH, we clamp it to 0.
        let last_sync_time = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(commit_time_seconds.max(0) as u64);

        // Calculate pending changes size in bytes
        let mut pending_size: u64 = 0;
        let statuses = repo
            .statuses(None)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;

        let workdir = repo
            .workdir()
            .unwrap_or_else(|| std::path::Path::new(&self.repo_path));

        for entry in statuses.iter() {
            if let Some(path_str) = entry.path() {
                let path = workdir.join(path_str);
                // Only count existing files (deleted files have 0 size contribution here)
                if let Ok(metadata) = std::fs::metadata(&path) {
                    pending_size += metadata.len();
                }
            }
        }

        debug!(
            commit = %commit.id(),
            pending_bytes = pending_size,
            "Last sync state retrieved"
        );

        Ok(SynchronisationReport {
            commit_name: commit.id().to_string(),
            last_sync_time,
            pending_changes: pending_size as usize,
        })
    }
}

impl From<git2::Error> for SynchronisationError {
    fn from(error: git2::Error) -> Self {
        SynchronisationError::SyncToolError(error.to_string())
    }
}
