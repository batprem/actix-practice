use futures_core::stream::Stream;
use std::pin::Pin;
use tokio::time::{sleep, Duration};
use async_stream::stream;
use tracing::{debug, info, instrument};

#[instrument]
pub async fn call_llm<'a>(query: &'a str) -> Pin<Box<dyn Stream<Item = String> + 'a>> {
    info!(query = query, "Starting LLM call");
    debug!("Preparing to generate stream response");
    
    let stream = stream! {
        let output = format!("Call LLM with query: \"{}\"", query);
        info!(output_length = output.len(), "Generated LLM output");
        debug!("Starting to stream characters");
        
        for (idx, c) in output.chars().enumerate() {
            sleep(Duration::from_millis(100)).await;
            debug!(char_index = idx, char = %c, "Streaming character");
            yield String::from(c);
        }
        
        info!("Finished streaming LLM response");
    };
    Box::pin(stream)
}

