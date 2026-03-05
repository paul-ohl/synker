use std::sync::Arc;

use crate::domain::ports::Synchronisation;
use crate::inbound::cli::daemon::timer;
use crate::inbound::server::setup::server;
use crate::inbound::server::state::AppState;

pub async fn run_server(state: AppState, addr: &str) {
    server(state, addr).await;
}

pub async fn run_daemon(_files_dir: &str, sync: Arc<dyn Synchronisation>, sync_delay: u64) {
    timer::watch(sync, sync_delay).await;
}
