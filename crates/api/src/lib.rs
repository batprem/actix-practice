use actix_web::{get, post, HttpResponse, Responder, web::Query};
use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use mock_llm::call_llm;
use tracing::{debug, error, info, instrument, span, warn, Level};
use futures::channel::mpsc;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use serde_json::json;

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



#[derive(Serialize, Deserialize)]
struct JustATest {
    x: u8,
}

#[derive(Serialize, Deserialize)]
struct Function1Result {
    goto: u8,
    just_a_test: JustATest,
    x: i32,
}

#[derive(Serialize, Deserialize)]
struct Event {
    event: String,
    content: String,
}

fn format_event(event: &Event) -> String {
    format!("data: {}\n\n", serde_json::to_string(event).unwrap())
}

async fn function1(tx: mpsc::Sender<Bytes>, goto: u8) -> Function1Result {
    let json_text = json!({ "goto": goto, "just_a_test": JustATest { x: 1 }, "x": 2 }).to_string();

    let mut result_buffer = String::new();
    for c in json_text.chars() {
        
        sleep(Duration::from_millis(100)).await;
        result_buffer.push(c);
        let event = format_event(&Event { event: "message".to_string(), content: result_buffer.clone() });
        let _ = tx.clone().try_send(Bytes::from(event));
    }
    serde_json::from_str::<Function1Result>(&result_buffer).unwrap()
}

async fn function2(tx: mpsc::Sender<Bytes>) {
    let _ = tx.clone().try_send(Bytes::from("data: This is function 2\n\n"));
    sleep(Duration::from_secs(1)).await;
}

async fn function3(tx: mpsc::Sender<Bytes>) {
    let _ = tx.clone().try_send(Bytes::from("data: This is function 3\n\n"));
    sleep(Duration::from_secs(1)).await;
}

#[derive(Deserialize)]
struct Params {
    goto: Option<u8>,
}
#[get("/stream")]
async fn stream_sse(params: Query<Params>) -> HttpResponse {
    let (mut tx, rx) = mpsc::channel::<Bytes>(16);

    actix_web::rt::spawn(async move {
        // --- Run function 1 ---
        let goto = params.goto.unwrap_or(2);
        let result = function1(tx.clone(), goto).await;

        // Stream the JSON result from function 1
        // let json_text = serde_json::to_string(&result).unwrap();
        // let _ = tx.try_send(Bytes::from(format!("data: {}\n\n", json_text)));

        // --- Route to next function ---
        match result.goto {
            2 => {
                function2(tx.clone()).await;
            },
            3 => {
                function3(tx.clone()).await;
            },
            _ => {
                let _ = tx.try_send(Bytes::from("data: Invalid goto\n\n"));
            }
        }

        // Close stream
        drop(tx);
    });

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .streaming(rx.map(
            |s| Ok::<Bytes, actix_web::Error>(s)
        ).map_err(|e| {
            error!(error = ?e, "Error sending bytes");
            actix_web::error::ErrorInternalServerError("Error sending bytes")
        }))
}
