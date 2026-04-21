mod handlers;
mod models;
mod security;

use axum::{
    routing::{get, post},
    Router,
};
use reqwest::Client;
use std::{env, sync::Arc};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use handlers::{extract_handler, status_handler, AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::status_handler,
        handlers::extract_handler,
    ),
    components(
        schemas(
            models::StatusResponse,
            models::ExtractRequest,
            models::ExtractResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "vision", description = "Document Vision API endpoints")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            )
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive("mtl_vision=debug".parse().unwrap()))
        .init();

    // Load .env file
    dotenvy::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let ollama_url = env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());

    let state = Arc::new(AppState {
        http_client: Client::new(),
        ollama_url,
    });

    // Create static file serving directory
    let static_dir = ServeDir::new("static");

    let api_routes = Router::new()
        .route("/status", get(status_handler))
        .route("/vision/extract", post(extract_handler))
        .with_state(state);

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback_service(static_dir)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
