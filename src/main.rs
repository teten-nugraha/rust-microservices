mod config;
mod handlers;
mod models;
mod repositories;
mod routes;
mod services;

use axum::{Json, Router, body::Body, http::Request, routing::get};

use serde::Serialize;

use routes::product_route::product_routes;

use config::{database, logging::init_logging};

use tower_http::request_id::MakeRequestUuid;
use tower_http::{
    request_id::{MakeRequestId, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing::log::Level::Trace;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "UP".to_string(),
        service: "inventory-service".to_string(),
    })
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Keep guard alive until application exists.
    let _log_guard = init_logging();

    info!("Starting inventory-service");

    // initialize databse connection pool
    let pool = database::connect()
        .await
        .expect("Database connection failed");

    info!("Database connection established");

    let app = Router::new()
        .route("/health", get(health))
        .merge(product_routes())
        .with_state(pool)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &axum::http::Request<axum::body::Body>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("unknown");

                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = %request.uri().path(),
                    request_id = %request_id,
                )
            },
        ));
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Failed to bind TCP Listener");

    info!(address = %address,"HTTP Server Listening on http://{}", address);

    axum::serve(listener, app).await.expect("Server Failed");
}
