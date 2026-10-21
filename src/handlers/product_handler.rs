use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::models::product::{CreateProductRequest, Product};
use crate::models::stock_movement::StockMovement;
use crate::repositories::product_repository::AdjustStockError;
use crate::services::product_service::ProductService;

use serde::Deserialize;
use sqlx::PgPool;

use tracing::{error, info, warn};

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

#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub sku: String,
    pub quantity: i32,
    pub reason: String,
}

pub async fn adjust_stock(
    State(pool): State<PgPool>,
    Json(request): Json<AdjustStockRequest>,
) -> Result<Json<Product>, StatusCode> {
    info!(
        sku = %request.sku,
        quantity = request.quantity,
        reason = %request.reason,
        "Stock Adjustment Requested"
    );

    if request.sku.trim().is_empty() || request.quantity == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    match ProductService::adjust_stock(&pool, &request.sku, request.quantity, &request.reason).await
    {
        Ok(product) => Ok(Json(product)),

        Err(AdjustStockError::NotFound) => Err(StatusCode::NOT_FOUND),

        Err(AdjustStockError::InvalidQuantity) => Err(StatusCode::BAD_REQUEST),

        Err(AdjustStockError::InsufficientStock) => Err(StatusCode::CONFLICT),

        Err(AdjustStockError::StockOverflow) => Err(StatusCode::CONFLICT),

        Err(AdjustStockError::Database(error)) => {
            eprintln!("Database error: {error:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_stock_movements(
    State(pool): State<PgPool>,
    Path(sku): Path<String>,
) -> Result<Json<Vec<StockMovement>>, StatusCode> {
    match ProductService::get_stock_movements(&pool, &sku).await {
        Ok(movements) => Ok(Json(movements)),
        Err(error) => {
            eprintln!("Error getting stock movements: {error:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
