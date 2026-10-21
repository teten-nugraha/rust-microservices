use sqlx::PgPool;

use crate::models::product::{CreateProductRequest, Product};

#[derive(Debug)]
pub enum AdjustStockError {
    NotFound,
    InvalidQuantity,
    InsufficientStock,
    StockOverflow,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for AdjustStockError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AdjustStockError::NotFound,
            _ => AdjustStockError::Database(err),
        }
    }
}

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

    pub async fn adjust_stock(
        pool: &PgPool,
        sku: &str,
        quantity: i32,
    ) -> Result<Product, AdjustStockError> {
        if quantity == 0 {
            return Err(AdjustStockError::InvalidQuantity);
        }

        // 1. Begin transaction
        let mut tx = pool.begin().await.map_err(AdjustStockError::Database)?;

        // 2. Lock the product row
        let current_stock: Option<i32> = sqlx::query_scalar(
            "SELECT stock
             FROM products
             WHERE sku = $1
             FOR UPDATE",
        )
        .bind(sku)
        .fetch_optional(&mut *tx)
        .await
        .map_err(AdjustStockError::Database)?;

        let current_stock = match current_stock {
            Some(stock) => stock,
            None => {
                tx.rollback().await.map_err(AdjustStockError::Database)?;

                return Err(AdjustStockError::NotFound);
            }
        };

        // 3. calculate using i63 to prevent i32 overflow
        let new_stock = i64::from(current_stock) + i64::from(quantity);

        if new_stock < 0 {
            tx.rollback().await.map_err(AdjustStockError::Database)?;

            return Err(AdjustStockError::InsufficientStock);
        }

        if new_stock > i64::from(i32::MAX) {
            tx.rollback().await.map_err(AdjustStockError::Database)?;

            return Err(AdjustStockError::StockOverflow);
        }

        let product = sqlx::query_as::<_, Product>(
            "UPDATE products
                SET stock = $1
                WHERE sku = $2
                RETURNING sku, name, price, stock",
        )
        .bind(new_stock as i32)
        .bind(sku)
        .fetch_one(&mut *tx)
        .await
        .map_err(AdjustStockError::Database)?;

        // 4. commit
        tx.commit().await.map_err(AdjustStockError::Database)?;

        Ok(product)
    }
}
