use wakey::discord::create_client;

#[tokio::main]
async fn main() {
    env_logger::init();
    create_client().await;
}
