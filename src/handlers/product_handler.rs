use axum::{Json, extract::Path, http::StatusCode};

use crate::models::product::{CreateProductRequest, Product};

// DUmmy product data
fn dummy_products() -> Vector<Product> {
    vec![
        Product {
            sku: "SKU001".to_string(),
            name: "T-Shirt Basic".to_string(),
            price: 99000,
            stock: 100,
        },
        Product {
            sku: "SKU002".to_string(),
            name: "Jeans Regular".to_string(),
            price: 249000,
            stock: 50,
        },
    ]
}

// GET /api/products
pub async fn get_products() -> Json<Vec<Product>> {
    Json(dummy_products())
}

// GET /api/products/{sku}
pub async fn get_product(Path(sku): Path<String>) -> Result<Json<Product>, StatusCode> {
    let product = dummy_products().into_iter().find(|p| p.sku == sku);

    match product {
        Some(product) => Ok(Json(product)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// POST  /api/products
pub async fn create_product(
    Json(request): Json<CreateProductRequest>,
) -> (StatusCode, Json<Product>) {

    let product = Product {
        sku: request.sku,
        name: request.name,
        price: request.price,
        stock: request.stock,
    }

    (StatusCode::CREATED, Json(product))
}