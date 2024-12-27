-- Create orders table

CREATE TABLE processed_block (
    id SERIAL PRIMARY KEY,
    height INTEGER NOT NULL,
    processed_at TIMESTAMP NOT NULL,
    success BOOLEAN NOT NULL
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    order_id BYTEA NOT NULL UNIQUE,
    "user" BYTEA NOT NULL,
    filler BYTEA NOT NULL,
    source_chain_selector BYTEA NOT NULL,
    destination_chain_selector BYTEA NOT NULL,
    sponsored BOOLEAN NOT NULL,
    primary_filler_deadline TIMESTAMPTZ NOT NULL,
    deadline TIMESTAMPTZ NOT NULL,
    call_recipient BYTEA NOT NULL,
    call_data BYTEA NOT NULL
);

-- Create tokens table
CREATE TABLE tokens (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    token_address BYTEA NOT NULL,
    token_id BIGINT NOT NULL,
    amount BIGINT NOT NULL
);