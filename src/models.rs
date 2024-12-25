use diesel::prelude::*;

#[derive(Queryable, Insertable, AsChangeset, Selectable, Debug)]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub id: i32,
    pub user: Vec<u8>,
    pub filler: Option<Vec<u8>>,
    pub source_chain_selector: i64,
    pub destination_chain_selector: i64,
    pub sponsored: bool,
    pub primary_filler_deadline: chrono::NaiveDateTime,
    pub deadline: chrono::NaiveDateTime,
    pub call_recipient: Option<Vec<u8>>,
    pub call_data: Option<Vec<u8>>,
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::schema::tokens)]
pub struct Token {
    pub id: i32,
    pub order_id: i32,
    pub token_address: Vec<u8>,
    pub token_id: i64,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::schema::processed_block)]
pub struct ProcessedBlock {
    pub id: i32,
    pub height: i32,
    pub processed_at: chrono::NaiveDateTime,
    pub success: bool,
}