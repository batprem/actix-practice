use std::io::Write;
use futures_util::pin_mut;
use futures::StreamExt;
use mock_llm::call_llm;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    info!("Starting mock LLM client");
    
    let query = "Hello, World!";
    info!(query = query, "Calling LLM service");
    
    let stream = call_llm(query).await;
    pin_mut!(stream);
    
    info!("Starting to consume stream");
    while let Some(item) = stream.next().await {
        print!("{}", item);
        std::io::stdout().flush().unwrap();
    }
    
    info!("Finished consuming stream");
}
