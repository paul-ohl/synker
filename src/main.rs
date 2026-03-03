#[tokio::main]
async fn main() {
    synker::inbound::server::setup::server().await;
}
