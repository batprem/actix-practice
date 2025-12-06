use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize OpenTelemetry tracing with OTLP exporter
///
/// This function sets up:
/// - OTLP exporter (configurable via OTEL_EXPORTER_OTLP_ENDPOINT env var)
/// - Tracing subscriber with console logging and OpenTelemetry layer
/// - Resource attributes including service name
///
/// # Errors
///
/// Returns an error if OpenTelemetry initialization fails
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize OpenTelemetry with OTLP exporter
    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(Resource::new(vec![opentelemetry::KeyValue::new(
                    SERVICE_NAME,
                    "actix-api",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Get tracer from the provider
    let tracer = tracer_provider.tracer("actix-api");

    // Initialize tracing subscriber with OpenTelemetry layer
    // Default to "info" level if RUST_LOG is not set
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("OpenTelemetry tracing initialized successfully");
    tracing::info!(
        otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string())
            .as_str(),
        "OTLP exporter configured"
    );

    Ok(())
}

/// Shutdown OpenTelemetry tracer provider
///
/// This should be called when the application is shutting down
/// to ensure all traces are properly exported
pub fn shutdown_tracing() {
    tracing::info!("Shutting down OpenTelemetry tracer provider");
    opentelemetry::global::shutdown_tracer_provider();
}

