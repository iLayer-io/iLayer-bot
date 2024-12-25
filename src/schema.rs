// @generated automatically by Diesel CLI.

diesel::table! {
    orders (id) {
        id -> Int4,
        user -> Bytea,
        filler -> Nullable<Bytea>,
        source_chain_selector -> Int8,
        destination_chain_selector -> Int8,
        sponsored -> Bool,
        primary_filler_deadline -> Timestamp,
        deadline -> Timestamp,
        call_recipient -> Nullable<Bytea>,
        call_data -> Nullable<Bytea>,
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
