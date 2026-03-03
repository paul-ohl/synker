#[tokio::main]
async fn main() {
    synker::inbound::cli::read_arguments::dispatch().await;
}
