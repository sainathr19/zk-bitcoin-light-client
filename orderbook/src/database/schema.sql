-- Orders table schema
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    source_asset VARCHAR(255) NOT NULL,
    destination_asset VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL, -- Using BIGINT for lowest denomination amounts
    status VARCHAR(50) NOT NULL DEFAULT 'created',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    proof_bytes JSONB,
    public_outputs JSONB
);

-- Create index on status for faster queries
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);

-- Create index on created_at for faster time-based queries
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);
