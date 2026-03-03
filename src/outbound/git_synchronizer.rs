use git2::Repository;

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
    fn add_all(&self, repo: &Repository) -> Result<(), SynchronisationError> {
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
    fn create_commit(
        &self,
        repo: &Repository,
        message: &str,
    ) -> Result<git2::Oid, SynchronisationError> {
        let signature = git2::Signature::now("AutoSync", &self.user_email)?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let parent_commit = repo.head()?.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )
        .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))
    }

    /// Performs a `git pull --rebase` from the configured remote's branch.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if fetching, rebasing, or committing rebased changes fails.
    /// If a conflict occurs, the rebase is aborted.
    fn pull_rebase(&self, repo: &Repository) -> Result<(), SynchronisationError> {
        let mut remote = repo.find_remote(&self.remote)?;
        remote.fetch(&[&self.branch], None, None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = fetch_head.peel_to_commit()?;
        let annotated_fetch_commit = repo.find_annotated_commit(fetch_commit.id())?;

        let mut rebase = repo.rebase(None, Some(&annotated_fetch_commit), None, None)?;
        let signature = git2::Signature::now("AutoSync", &self.user_email)?;

        while let Some(op) = rebase.next() {
            match op {
                Ok(_) => {
                    if repo.index()?.has_conflicts() {
                        rebase.abort()?;
                        return Err(SynchronisationError::FileConflict(
                            "Merge conflict detected during rebase".to_string(),
                        ));
                    }
                    rebase.commit(None, &signature, None)?;
                }
                Err(e) => {
                    rebase.abort()?;
                    return Err(SynchronisationError::SyncToolError(e.to_string()));
                }
            }
        }
        rebase.finish(None)?;
        Ok(())
    }

    /// Pushes the local branch to the configured remote.
    ///
    /// # Errors
    /// Returns a `SynchronisationError` if the push operation fails.
    fn push_to_origin(&self, repo: &Repository) -> Result<(), SynchronisationError> {
        let mut remote = repo.find_remote(&self.remote)?;
        let refspec = format!("refs/heads/{}:refs/heads/{}", self.branch, self.branch);
        remote.push(&[&refspec], None)?;
        Ok(())
    }
}

impl ports::Synchronisation for GitSynchronizer {
    /// Checks if there are any staged or unstaged changes in the repository.
    fn register_changes(&self) -> Result<bool, SynchronisationError> {
        git2::Repository::discover(&self.repo_path)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))
            .and_then(|repo| {
                let statuses = repo
                    .statuses(None)
                    .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;
                Ok(!statuses.is_empty())
            })
    }

    /// Synchronizes the local repository with the configured remote branch.
    ///
    /// The process includes:
    /// 1. Staging all changes (`git add -A`)
    /// 2. Creating an auto-generated commit
    /// 3. Fetching and rebasing from the remote branch
    /// 4. Pushing to the remote branch
    fn synchronise(&self) -> Result<SynchronisationReport, SynchronisationError> {
        let repo = Repository::discover(&self.repo_path)?;

        self.add_all(&repo)?;

        let commit_message = self.generate_commit_message(&repo)?;
        let commit_id = self.create_commit(&repo, &commit_message)?;

        self.pull_rebase(&repo)?;
        self.push_to_origin(&repo)?;

        Ok(SynchronisationReport {
            commit_name: commit_id.to_string(),
            last_sync_time: Some(std::time::SystemTime::now()),
            last_sync_duration: None,
            pending_changes: 0,
        })
    }

    /// Provides a report on the current synchronization state.
    fn is_synchronised(&self) -> Result<SynchronisationReport, SynchronisationError> {
        let repo = Repository::discover(&self.repo_path)
            .map_err(|e| SynchronisationError::SyncToolError(e.to_string()))?;

        let head = repo.head()?.peel_to_commit()?;
        let statuses = repo.statuses(None)?;

        Ok(SynchronisationReport {
            commit_name: head.id().to_string(),
            last_sync_time: None,
            last_sync_duration: None,
            pending_changes: statuses.len(),
        })
    }
}

impl From<git2::Error> for SynchronisationError {
    fn from(error: git2::Error) -> Self {
        SynchronisationError::SyncToolError(error.to_string())
    }
}
