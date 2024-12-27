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
    filler BYTEA,
    source_chain_selector BIGINT NOT NULL,
    destination_chain_selector BIGINT NOT NULL,
    sponsored BOOLEAN NOT NULL,
    primary_filler_deadline TIMESTAMP NOT NULL,
    deadline TIMESTAMP NOT NULL,
    call_recipient BYTEA,
    call_data BYTEA
);

-- Create tokens table
CREATE TABLE tokens (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    token_address BYTEA NOT NULL,
    token_id BIGINT NOT NULL,
    amount BIGINT NOT NULL
);