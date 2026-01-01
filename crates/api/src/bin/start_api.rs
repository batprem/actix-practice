use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
use reqwest::Client;
use api::{hello, echo, manual_hello, stream_llm, middlewares};
use bytes::Bytes;
use tracing_actix_web::TracingLogger;
use tracing::info;
use std::str::FromStr;
use futures::StreamExt;


async fn fallback_proxy(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Client>,
) -> HttpResponse {
    let method = reqwest::Method::from_str(req.method().as_str()).unwrap_or(reqwest::Method::GET);
    let path = req
        .uri()
        .path_and_query()
        .map(|p| p.to_string())
        .unwrap_or_default();
    let url = format!("http://localhost:8000{}", path);

    let mut forward = client.request(method, &url);

    for (key, value) in req.headers() {
        forward = forward.header(key.as_str(), value.to_str().unwrap_or_default());
    }

    let response = forward.body(body).send().await;

    match response {
        Ok(r) => {
            let status_code = r.status().as_u16();
            let content_type = r
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            // Extract MIME type (before semicolon) and trim whitespace
            let mime_type = content_type.split(';').next().unwrap_or("").trim();
            if mime_type == "text/event-stream" {
                info!("Streaming response");
                return HttpResponse::Ok()
                    .append_header(("Content-Type", content_type.as_str()))
                    .streaming(r.bytes_stream().map(|chunk| {
                        chunk.map(|b| Bytes::from(b)).map_err(|e| {
                            actix_web::error::ErrorInternalServerError(format!(
                                "Stream error: {}",
                                e
                            ))
                        })
                    }));
            }
            let bytes = r.bytes().await.unwrap_or_default();

            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status_code)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
            )
            .insert_header(("Content-Type", content_type.as_str()))
            .body(bytes)
        }
        Err(_) => HttpResponse::BadGateway().body("Fallback to Python failed"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize OpenTelemetry tracing
    middlewares::otel::init_tracing()
        .expect("Failed to initialize tracing");

    info!("Starting Actix web server on 127.0.0.1:8082");

    let client = Client::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(hello)
                    .service(echo)
                    .route("/hey", web::get().to(manual_hello))
                    .service(stream_llm),
            )
            .default_service(web::route().to(fallback_proxy))
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await?;

    // Shutdown OpenTelemetry tracer provider
    info!("Shutting down server");
    middlewares::otel::shutdown_tracing();

    Ok(())
}
