use sqlx::PgPool;

use crate::models::product::{CreateProductRequest, Product};

pub struct ProductRepository;

impl ProductRepository {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
                SELECT sku, name, price, stock FROM products ORDER BY sku ASC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_sku(pool: &PgPool, sku: &str) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
                SELECT sku, name, price, stock FROM products WHERE sku = $1
            "#,
        )
        .bind(sku)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        request: &CreateProductRequest,
    ) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>(
            r#"
                INSERT INTO products(sku, name, price, stock) VALUES($1, $2, $3, $4) RETURNING sku, name, price, stock
            "#,
        )
        .bind(&request.sku)
        .bind(&request.name)
        .bind(request.price)
        .bind(request.stock)
        .fetch_one(pool)
        .await
    }
}
