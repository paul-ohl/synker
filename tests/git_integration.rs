use git2::Repository;
use std::path::Path;
use synker::domain::ports::Synchronisation;
use synker::domain::services::synchronisation::{SynchronisationError, SynchronisationReport};
use synker::outbound::git_synchronizer::GitSynchronizer;
use tempfile::tempdir;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Initialises a local git repo with an initial empty commit so HEAD is valid.
/// Returns the repo and the branch name used (determined by git defaults).
fn setup_repo_with_initial_commit(path: &Path) -> (Repository, String) {
    // Explicitly set the initial branch to "master" via init options so tests
    // are predictable regardless of the user's global git config.
    let mut init_opts = git2::RepositoryInitOptions::new();
    init_opts.initial_head("master");
    let repo = Repository::init_opts(path, &init_opts).unwrap();

    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
    {
        let mut index = repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }
    (repo, "master".to_string())
}

/// Adds a remote pointing to the bare repo and pushes the initial commit.
fn add_remote_and_push_initial(local_repo: &Repository, remote_path: &Path, branch: &str) {
    local_repo
        .remote("origin", remote_path.to_str().unwrap())
        .unwrap();
    let mut remote = local_repo.find_remote("origin").unwrap();
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    remote.push(&[&refspec], None).unwrap();
}

/// Counts the number of commits reachable from a given branch in a repository.
fn count_commits(repo: &Repository, branch: &str) -> usize {
    let refname = format!("refs/heads/{branch}");
    let reference = repo.find_reference(&refname).unwrap();
    let commit = reference.peel_to_commit().unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push(commit.id()).unwrap();
    revwalk.count()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_get_last_sync_first_time_sync() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Set up local repo with initial commit but do NOT push (no remote ref yet)
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    // Add remote but do NOT push yet — so no refs/remotes/origin/master exists
    local_repo
        .remote("origin", bare_dir.path().to_str().unwrap())
        .unwrap();
    // Initialize bare repo so remote URL is valid
    Repository::init_bare(bare_dir.path()).unwrap();

    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch,
        work_dir.path().to_str().unwrap().to_string(),
        "test@example.com".to_string(),
    );

    let result = sync.get_last_sync();
    assert!(
        matches!(result, Err(SynchronisationError::FirstTimeSync)),
        "Expected FirstTimeSync but got: {:?}",
        result
    );
}

#[test]
fn test_synchronise_creates_commit_and_pushes() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Create bare remote
    Repository::init_bare(bare_dir.path()).unwrap();

    // Create local repo with initial commit and push it
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    add_remote_and_push_initial(&local_repo, bare_dir.path(), &branch);

    // Write a new file to the working directory
    let new_file = work_dir.path().join("hello.txt");
    std::fs::write(&new_file, "Hello, world!").unwrap();

    // Run synchronise
    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch.clone(),
        work_dir.path().to_str().unwrap().to_string(),
        "sync@example.com".to_string(),
    );

    let result = sync.synchronise();
    assert!(result.is_ok(), "synchronise() failed: {:?}", result.err());

    // Open bare repo and verify a commit was pushed
    let bare_repo = Repository::open(bare_dir.path()).unwrap();
    let head = bare_repo
        .find_reference(&format!("refs/heads/{branch}"))
        .unwrap();
    let commit = head.peel_to_commit().unwrap();

    // The latest commit message should start with "Auto-sync:"
    assert!(
        commit.message().unwrap_or("").starts_with("Auto-sync:"),
        "Expected commit message to start with 'Auto-sync:', got: {:?}",
        commit.message()
    );
}

#[test]
fn test_synchronise_no_changes_skips_commit() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Create bare remote
    Repository::init_bare(bare_dir.path()).unwrap();

    // Create local repo with initial commit and push it
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    add_remote_and_push_initial(&local_repo, bare_dir.path(), &branch);

    // Verify initial commit count is 1
    let bare_repo = Repository::open(bare_dir.path()).unwrap();
    assert_eq!(count_commits(&bare_repo, &branch), 1);

    // Run synchronise without any new changes
    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch.clone(),
        work_dir.path().to_str().unwrap().to_string(),
        "sync@example.com".to_string(),
    );

    let result = sync.synchronise();
    assert!(result.is_ok(), "synchronise() failed: {:?}", result.err());

    // Commit count should still be 1 (no new commit created)
    let bare_repo2 = Repository::open(bare_dir.path()).unwrap();
    assert_eq!(
        count_commits(&bare_repo2, &branch),
        1,
        "Expected commit count to remain 1 when there are no changes"
    );
}

#[test]
fn test_get_last_sync_after_sync() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Create bare remote
    Repository::init_bare(bare_dir.path()).unwrap();

    // Create local repo with initial commit and push it
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    add_remote_and_push_initial(&local_repo, bare_dir.path(), &branch);

    // Write a file and synchronise
    let new_file = work_dir.path().join("notes.md");
    std::fs::write(&new_file, "# Notes").unwrap();

    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch,
        work_dir.path().to_str().unwrap().to_string(),
        "sync@example.com".to_string(),
    );

    sync.synchronise().unwrap();

    // Now get_last_sync should succeed and report 0 pending changes
    let report = sync
        .get_last_sync()
        .expect("get_last_sync should succeed after a sync");
    assert_eq!(
        report.pending_changes, 0,
        "Expected 0 pending changes after a full sync"
    );
}

#[test]
fn test_add_all_stages_files() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Create bare remote
    Repository::init_bare(bare_dir.path()).unwrap();

    // Create local repo with initial commit and push it
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    add_remote_and_push_initial(&local_repo, bare_dir.path(), &branch);

    // Write a file
    let staged_file = work_dir.path().join("staged.txt");
    std::fs::write(&staged_file, "staged content").unwrap();

    // Synchronise (which internally calls add_all + create_commit)
    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch.clone(),
        work_dir.path().to_str().unwrap().to_string(),
        "sync@example.com".to_string(),
    );

    sync.synchronise().unwrap();

    // After sync, the file should appear in the latest commit in the bare repo
    let bare_repo = Repository::open(bare_dir.path()).unwrap();
    let head_commit = bare_repo
        .find_reference(&format!("refs/heads/{branch}"))
        .unwrap()
        .peel_to_commit()
        .unwrap();
    let tree = head_commit.tree().unwrap();

    // Check that "staged.txt" is present in the tree
    let entry = tree.get_name("staged.txt");
    assert!(
        entry.is_some(),
        "staged.txt should appear in the commit tree after synchronise()"
    );
}

#[test]
fn test_get_last_sync_with_pending_changes() {
    let work_dir = tempdir().unwrap();
    let bare_dir = tempdir().unwrap();

    // Create bare remote
    Repository::init_bare(bare_dir.path()).unwrap();

    // Create local repo with initial commit and push it
    let (local_repo, branch) = setup_repo_with_initial_commit(work_dir.path());
    add_remote_and_push_initial(&local_repo, bare_dir.path(), &branch);

    // Write a new file WITHOUT syncing — this creates a pending change
    let pending_file = work_dir.path().join("pending.txt");
    std::fs::write(&pending_file, "not yet synced").unwrap();

    let sync = GitSynchronizer::new(
        "origin".to_string(),
        branch.clone(),
        work_dir.path().to_str().unwrap().to_string(),
        "test@example.com".to_string(),
    );

    // get_last_sync should succeed (remote ref exists) and show pending_changes > 0
    let report = sync
        .get_last_sync()
        .expect("get_last_sync should succeed after initial push");

    assert!(
        report.pending_changes > 0,
        "Expected pending_changes > 0 with an unsynced file, got: {}",
        report.pending_changes
    );
}

#[test]
fn test_git2_error_converts_to_synchronisation_error() {
    // Exercise the From<git2::Error> impl by using a GitSynchronizer
    // with an invalid repo path — this triggers the git2 error path.
    let sync = GitSynchronizer::new(
        "origin".to_string(),
        "master".to_string(),
        "/nonexistent/path/that/does/not/exist".to_string(),
        "test@example.com".to_string(),
    );

    let result = sync.synchronise();
    assert!(
        result.is_err(),
        "Expected error when using nonexistent repo path"
    );
    // The error should be a SyncToolError (via From<git2::Error>)
    assert!(
        matches!(result, Err(SynchronisationError::SyncToolError(_))),
        "Expected SyncToolError, got something else"
    );
}

// Suppress unused import warning when SynchronisationReport isn't used directly in tests
#[allow(dead_code)]
fn _use_report(_r: SynchronisationReport) {}
