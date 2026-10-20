CREATE TABLE products (
    sku VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price BIGINT NOT NULL,
    stock INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_price_positive
        CHECK (price > 0),

    CONSTRAINT chk_stock_non_negative
        CHECK (stock >= 0)
);