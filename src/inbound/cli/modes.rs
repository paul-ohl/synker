use crate::inbound::server::setup::server;

pub async fn run_server() {
    server().await;
}

pub async fn run_daemon() {
    loop {}
}
