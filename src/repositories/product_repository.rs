use sqlx::PgPool;

use crate::models::product::{CreateProductRequest, Product};
use crate::models::stock_movement::StockMovement;

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
        reason: &str,
    ) -> Result<Product, AdjustStockError> {
        if quantity == 0 || reason.trim().is_empty() {
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

        let stock_before = match current_stock {
            Some(stock) => stock,
            None => {
                tx.rollback().await.map_err(AdjustStockError::Database)?;

                return Err(AdjustStockError::NotFound);
            }
        };

        // 3. calculate using i63 to prevent i32 overflow
        let calculated = i64::from(stock_before) + i64::from(quantity);

        if calculated < 0 {
            tx.rollback().await.map_err(AdjustStockError::Database)?;

            return Err(AdjustStockError::InsufficientStock);
        }

        if calculated > i64::from(i32::MAX) {
            tx.rollback().await.map_err(AdjustStockError::Database)?;

            return Err(AdjustStockError::StockOverflow);
        }

        let stock_after = calculated as i32;

        let product = sqlx::query_as::<_, Product>(
            r#"
                UPDATE products
                SET stock = $1
                WHERE sku = $2
                RETURNING sku, name, price, stock
                "#,
        )
        .bind(stock_after)
        .bind(sku)
        .fetch_one(&mut *tx)
        .await
        .map_err(AdjustStockError::Database)?;

        // 4 Simpan audit trail dalam transaksi yang sama
        sqlx::query(
            r#"
                INSERT INTO stock_movements (sku, quantity, stock_before, stock_after, reason)
                VALUES ($1, $2, $3, $4, $5)
                "#,
        )
        .bind(sku)
        .bind(quantity)
        .bind(stock_before)
        .bind(stock_after)
        .bind(reason)
        .execute(&mut *tx)
        .await
        .map_err(AdjustStockError::Database)?;

        // 5. commit
        tx.commit().await.map_err(AdjustStockError::Database)?;

        Ok(product)
    }

    pub async fn find_stock_movements(
        pool: &PgPool,
        sku: &str,
    ) -> Result<Vec<StockMovement>, sqlx::Error> {
        sqlx::query_as::<_, StockMovement>(
            r#"
                SELECT
                    id,
                    sku,
                    quantity,
                    stock_before,
                    stock_after,
                    reason,
                    to_char(created_at AT TIME ZONE 'UTC',
                        'YYYY-MM-DD HH24:MI:SS'
                    ) || 'Z' AS created_at
                FROM stock_movements
                WHERE sku = $1
                ORDER BY created_at DESC, id  DESC
                LIMIT 100
                "#,
        )
        .bind(sku)
        .fetch_all(pool)
        .await
    }
}
