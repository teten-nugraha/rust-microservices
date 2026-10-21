mod config;
mod handlers;
mod models;
mod repositories;
mod routes;
mod services;

use axum::{Json, Router, routing::get};

use serde::Serialize;

use routes::product_route::product_routes;

use config::database;

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

    // initialize databse connection pool
    let pool = database::connect()
        .await
        .expect("Database connection failed");

    let app = Router::new()
        .route("/health", get(health))
        .merge(product_routes())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind server");

    println!("Inventory Service running on :8080");

    axum::serve(listener, app).await.expect("Server Failed");
}
