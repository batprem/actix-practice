use actix_web::{get, post, HttpResponse, Responder};
use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use mock_llm::call_llm;
use tracing::{debug, error, info, instrument, span, warn, Level};

pub mod middlewares;

#[get("/")]
#[instrument]
pub async fn hello() -> impl Responder {
    info!("Hello endpoint called");
    debug!("Processing hello request");
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
#[instrument]
pub async fn echo(req_body: String) -> impl Responder {
    info!(body_length = req_body.len(), "Echo endpoint called");
    debug!(body_preview = req_body.chars().take(50).collect::<String>().as_str(), "Echo request body preview");
    
    if req_body.is_empty() {
        warn!("Received empty echo request body");
    }
    
    HttpResponse::Ok().body(req_body)
}

#[instrument]
pub async fn manual_hello() -> impl Responder {
    info!("Manual hello endpoint called");
    debug!("Processing manual hello request");
    HttpResponse::Ok().body("Hey there2!")
}

#[get("/stream-llm")]
#[instrument]
pub async fn stream_llm() -> impl Responder {
    let span = span!(Level::INFO, "stream_llm_handler");
    let _enter = span.enter();
    
    let query = "Hello, World!";
    info!(query = query, "Starting LLM stream");
    debug!("Preparing to call LLM service");
    
    let stream = call_llm(query).await;
    info!("LLM service responded successfully");
    
    debug!("Converting stream to bytes");
    let bytes_stream = stream.map(
        |s| Ok::<Bytes, actix_web::Error>(Bytes::from(s))
    ).map_err(|e| {
        error!(error = ?e, "Error converting stream to bytes");
        actix_web::error::ErrorInternalServerError("Error converting stream to bytes")
    });
    
    info!("LLM stream created successfully, sending response");
    HttpResponse::Ok()
    .append_header(("Content-Type", "text/event-stream"))
    .append_header(("Cache-Control", "no-cache"))
    .streaming(bytes_stream)
}   