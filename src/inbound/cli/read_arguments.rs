use std::env;
use std::sync::Arc;

use crate::domain::ports::Synchronisation;
use crate::inbound::cli::modes::{run_daemon, run_server};
use crate::inbound::server::state::AppState;

/// All pre-built dependencies needed to run any mode.
pub struct Deps {
    pub state: AppState,
    pub addr: String,
    pub files_dir: String,
    pub sync: Arc<dyn Synchronisation>,
}

pub async fn dispatch(deps: Deps) {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("server") => run_server(deps.state, &deps.addr).await,
        Some("daemon") => run_daemon(&deps.files_dir, deps.sync).await,
        Some(unknown) => {
            eprintln!("Unknown mode: '{}'. Valid modes are: server, daemon", unknown);
            std::process::exit(1);
        }
        None => {
            let program = args.first().map(String::as_str).unwrap_or("synker");
            eprintln!("Usage: {} <server|daemon>", program);
            std::process::exit(1);
        }
    }
}
