use std::sync::Arc;

use dotenvy::dotenv;
use synker::{
    domain::logic::file_manager::FileManagerLogic,
    inbound::{
        cli::read_arguments::{Deps, dispatch},
        server::state::AppState,
    },
    outbound::{file_system::FsFileManager, git_synchronizer::GitSynchronizer},
};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialise the tracing subscriber.
    // Log level is controlled via the RUST_LOG environment variable.
    // Example: RUST_LOG=info  or  RUST_LOG=synker=debug,tower_http=trace
    // Defaults to "info" when RUST_LOG is not set.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting synker");

    let port = std::env::var("PORT").expect("PORT environment variable not set");
    let files_dir = std::env::var("FILES_DIR").expect("FILES_DIR environment variable not set");
    let repo_path = files_dir.clone();
    let git_remote = std::env::var("GIT_REMOTE").expect("GIT_REMOTE environment variable not set");
    let git_branch = std::env::var("GIT_BRANCH").expect("GIT_BRANCH environment variable not set");
    let git_user_email =
        std::env::var("GIT_USER_EMAIL").expect("GIT_USER_EMAIL environment variable not set");
    let sync_delay = std::env::var("SYNC_DELAY_HOURS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(24);

    info!(
        port,
        files_dir,
        git_remote,
        git_branch,
        git_user_email,
        sync_delay_hours = sync_delay,
        "Configuration loaded"
    );

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
        sync_delay,
    };

    dispatch(deps).await;
}
