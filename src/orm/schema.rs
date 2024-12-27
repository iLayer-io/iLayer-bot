// @generated automatically by Diesel CLI.

diesel::table! {
    orders (id) {
        id -> Int4,
        order_id -> Bytea,
        user -> Bytea,
        filler -> Bytea,
        source_chain_selector -> Bytea,
        destination_chain_selector -> Bytea,
        sponsored -> Bool,
        primary_filler_deadline -> Timestamptz,
        deadline -> Timestamptz,
        call_recipient -> Bytea,
        call_data -> Bytea,
    }
}

diesel::table! {
    processed_block (id) {
        id -> Int4,
        height -> Int4,
        processed_at -> Timestamp,
        success -> Bool,
    }
}

diesel::table! {
    tokens (id) {
        id -> Int4,
        order_id -> Int4,
        token_address -> Bytea,
        token_id -> Int8,
        amount -> Int8,
    }
}

diesel::joinable!(tokens -> orders (order_id));

diesel::allow_tables_to_appear_in_same_query!(
    orders,
    processed_block,
    tokens,
);
