use std::sync::Arc;

use dotenvy::dotenv;
use std::env;

use crate::inbound::cli::daemon::watcher;
use crate::inbound::server::setup::server;
use crate::outbound::git_synchronizer::GitSynchronizer;

pub async fn run_server() {
    server().await;
}

pub async fn run_daemon() {
    dotenv().ok();
    let files_dir = env::var("FILES_DIR").expect("FILES_DIR environment variable not set");
    let repo_path = env::var("REPO_PATH").expect("REPO_PATH environment variable not set");
    let git_remote = env::var("GIT_REMOTE").expect("GIT_REMOTE environment variable not set");
    let git_branch = env::var("GIT_BRANCH").expect("GIT_BRANCH environment variable not set");
    let git_user_email =
        env::var("GIT_USER_EMAIL").expect("GIT_USER_EMAIL environment variable not set");

    let sync = Arc::new(GitSynchronizer::new(
        git_remote,
        git_branch,
        repo_path,
        git_user_email,
    ));

    if let Err(e) = watcher::watch(&files_dir, sync).await {
        eprintln!("Watcher error: {:?}", e);
    }
}
