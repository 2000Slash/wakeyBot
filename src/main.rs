use wakey::discord::create_client;



#[tokio::main]
async fn main() {
    create_client().await;
}
