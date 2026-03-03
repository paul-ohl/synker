use std::env;

use super::modes::{run_daemon, run_server};

pub async fn dispatch() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("server") => run_server().await,
        Some("daemon") => run_daemon().await,
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
