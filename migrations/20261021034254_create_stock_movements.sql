CREATE TABLE stock_movements (

    id BIGSERIAL PRIMARY KEY ,
    sku VARCHAR(50) NOT NULL REFERENCES products(sku),
    quantity INTEGER NOT NULL CHECK (quantity <> 0),
    stock_before INTEGER NOT NULL CHECK (stock_before >= 0),
    stock_after INTEGER NOT NULL CHECK (stock_after >= 0),
    reason VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()

);

CREATE INDEX idx_stock_movements_sku_created_at
    ON stock_movements (sku, created_at DESC);