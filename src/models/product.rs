use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub sku: String,
    pub name: String, 
    pub price: i64,
    pub stock : i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub price: i63,
    pub stock: i32,
}