use crate::models::product::Product;

pub struct InventoryService;

impl InventoryService {
    pub fn is_available(product: &Product) -> bool {
        product.stock > 0
    }

    pub fn adjust_stock(product: &mut Product, quantity: i32) -> Result<(), String> {
        let new_stock = product
            .stock
            .checked_add(quantity)
            .ok_or("Stock calculation overflow")?;

        if new_stock < 0 {
            return Err("Insufficien stock".to_string());
        }

        product.stock = new_stock;

        Ok(())
    }
}
