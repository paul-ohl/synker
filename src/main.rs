use std::sync::Arc;

use dotenvy::dotenv;
use synker::domain::logic::file_manager::FileManagerLogic;
use synker::inbound::cli::read_arguments::{Deps, dispatch};
use synker::inbound::server::state::AppState;
use synker::outbound::file_system::FsFileManager;
use synker::outbound::git_synchronizer::GitSynchronizer;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = std::env::var("PORT").expect("PORT environment variable not set");
    let files_dir = std::env::var("FILES_DIR").expect("FILES_DIR environment variable not set");
    let repo_path = std::env::var("REPO_PATH").expect("REPO_PATH environment variable not set");
    let git_remote = std::env::var("GIT_REMOTE").expect("GIT_REMOTE environment variable not set");
    let git_branch = std::env::var("GIT_BRANCH").expect("GIT_BRANCH environment variable not set");
    let git_user_email =
        std::env::var("GIT_USER_EMAIL").expect("GIT_USER_EMAIL environment variable not set");

    // File manager adapter (server mode)
    let fs_adapter =
        FsFileManager::new(&files_dir).expect("Failed to initialise filesystem adapter");
    let file_manager = Arc::new(FileManagerLogic::new(Arc::new(fs_adapter)));
    let state = AppState { file_manager };

    // Synchronisation adapter (daemon mode)
    let sync = Arc::new(GitSynchronizer::new(
        git_remote,
        git_branch,
        repo_path,
        git_user_email,
    ));

    let deps = Deps {
        state,
        addr: format!("0.0.0.0:{}", port),
        files_dir,
        sync,
    };

    dispatch(deps).await;
}
