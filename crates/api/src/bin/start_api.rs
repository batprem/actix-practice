use actix_web::{web, App, HttpServer};
use api::{hello, echo, manual_hello, stream_llm, middlewares};
use tracing_actix_web::TracingLogger;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize OpenTelemetry tracing
    middlewares::otel::init_tracing()
        .expect("Failed to initialize tracing");

    info!("Starting Actix web server on 127.0.0.1:8082");

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(hello)
                    .service(echo)
                    .route("/hey", web::get().to(manual_hello))
                    .service(stream_llm),
            )
    })
    .bind(("127.0.0.1", 8082))?
    .run()
    .await?;

    // Shutdown OpenTelemetry tracer provider
    info!("Shutting down server");
    middlewares::otel::shutdown_tracing();

    Ok(())
}
