use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub sku: String,
    pub name: String,
    pub price: i64,
    pub stock: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub price: i64,
    pub stock: i32,
}
