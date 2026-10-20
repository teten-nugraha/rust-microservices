use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use sqlx::PgPool;

use crate::models::product::{CreateProductRequest, Product};
use crate::services::product_service::ProductService;

// GET /api/products
pub async fn get_products(State(pool): State<PgPool>) -> Result<Json<Vec<Product>>, StatusCode> {
    let products = ProductService::get_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(products))
}

// GET /api/products/{sku}
pub async fn get_product(
    State(pool): State<PgPool>,
    Path(sku): Path<String>,
) -> Result<Json<Product>, StatusCode> {
    let product = ProductService::get_by_sku(&pool, &sku)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match product {
        Some(product) => Ok(Json(product)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// POST  /api/products
pub async fn create_product(
    State(pool): State<PgPool>,
    Json(request): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    if request.sku.trim().is_empty()
        || request.name.trim().is_empty()
        || request.price <= 0
        || request.stock < 0
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let product = ProductService::create(&pool, &request)
        .await
        .map_err(|error| match &error {
            sqlx::Error::Database(db_error) if db_error.is_unique_violation() => {
                StatusCode::CONFLICT
            }

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok((StatusCode::CREATED, Json(product)))
}
