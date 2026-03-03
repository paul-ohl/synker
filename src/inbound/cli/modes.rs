use std::sync::Arc;

use crate::domain::ports::Synchronisation;
use crate::inbound::cli::daemon::watcher;
use crate::inbound::server::setup::server;
use crate::inbound::server::state::AppState;

pub async fn run_server(state: AppState, addr: &str) {
    server(state, addr).await;
}

pub async fn run_daemon(files_dir: &str, sync: Arc<dyn Synchronisation>) {
    if let Err(e) = watcher::watch(files_dir, sync).await {
        eprintln!("Watcher error: {:?}", e);
    }
}
