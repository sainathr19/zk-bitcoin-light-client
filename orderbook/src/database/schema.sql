-- Drop existing orders table if it exists (for migration)
DROP TABLE IF EXISTS orders CASCADE;

-- Orders table schema
CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_asset VARCHAR(255) NOT NULL,
    destination_asset VARCHAR(255) NOT NULL,
    source_address VARCHAR(255) NOT NULL,
    destination_address VARCHAR(255) NOT NULL,
    deposit_address VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL, -- Using BIGINT for lowest denomination amounts
    status VARCHAR(50) NOT NULL DEFAULT 'created',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    proof_bytes JSONB,
    public_values JSONB
);

-- Create index on status for faster queries
CREATE INDEX idx_orders_status ON orders(status);

-- Create index on created_at for faster time-based queries
CREATE INDEX idx_orders_created_at ON orders(created_at);
