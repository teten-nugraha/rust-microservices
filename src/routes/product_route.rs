use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::handlers::product_handler::{adjust_stock, create_product, get_product, get_products};

pub fn product_routes() -> Router<PgPool> {
    Router::new()
        .route("/api/products", get(get_products).post(create_product))
        .route("/api/products/{sku}", get(get_product))
        .route("/api/inventory/adjust", post(adjust_stock))
}
