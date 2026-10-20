use sqlx::PgPool;

use crate::{
    models::product::{CreateProductRequest, Product},
    repositories::product_repository::ProductRepository,
};

pub struct ProductService;

impl ProductService {
    pub async fn get_all(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
        ProductRepository::find_all(pool).await
    }

    pub async fn get_by_sku(pool: &PgPool, sku: &str) -> Result<Option<Product>, sqlx::Error> {
        ProductRepository::find_by_sku(pool, sku).await
    }

    pub async fn create(
        pool: &PgPool,
        request: &CreateProductRequest,
    ) -> Result<Product, sqlx::Error> {
        ProductRepository::create(pool, request).await
    }
}
