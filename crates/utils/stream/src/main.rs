use tokio;
use futures_core::stream::Stream;
use std::pin::Pin;
use std::io::Write;
use async_stream::stream;
use futures_util::pin_mut;
use futures::StreamExt;
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[instrument]
async fn run_stream() -> Pin<Box<dyn Stream<Item = String>>> {
    info!("Creating stream");
    let stream = stream! {
        yield String::from("Hello");
        yield String::from("World");
    };
    Box::pin(stream)
}

#[tokio::main]
async fn main() {
    init_tracing();
    info!("Starting stream utility");
    
    let stream = run_stream().await;
    pin_mut!(stream);
    
    info!("Consuming stream");
    while let Some(item) = stream.next().await {
        println!("{}", item);
        std::io::stdout().flush().unwrap();
    }
    
    info!("Finished consuming stream");
}
