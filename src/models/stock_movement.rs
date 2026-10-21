use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct StockMovement {
    pub id: i64,
    pub sku: String,
    pub quantity: i32,
    pub stock_before: i32,
    pub stock_after: i32,
    pub reason: String,
    pub created_at: String,
}
