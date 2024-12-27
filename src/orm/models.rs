use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default,Selectable,Debug,Queryable,Identifiable,Insertable,Serialize,Deserialize)]
#[diesel(table_name = crate::orm::schema::orders)]
#[diesel(primary_key(id))]
pub struct Order {
    pub id: i32,
    pub user: Vec<u8>,
    pub order_id: Vec<u8>,
    pub filler: Vec<u8>,
    pub source_chain_selector: Vec<u8>,
    pub destination_chain_selector: Vec<u8>,
    pub sponsored: bool,
    pub primary_filler_deadline: chrono::DateTime<chrono::Utc>,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub call_recipient: Vec<u8>,
    pub call_data: Vec<u8>,
}


#[derive(Debug,Insertable)]
#[diesel(table_name = crate::orm::schema::orders)]
pub struct NewOrder {
    pub user: Vec<u8>,
    pub order_id: Vec<u8>,
    pub filler: Vec<u8>,
    pub source_chain_selector: Vec<u8>,
    pub destination_chain_selector: Vec<u8>,
    pub sponsored: bool,
    pub primary_filler_deadline: chrono::DateTime<chrono::Utc>,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub call_recipient: Vec<u8>,
    pub call_data: Vec<u8>,
}


#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::orm::schema::tokens)]
pub struct Token {
    pub id: i32,
    pub order_id: i32,
    pub token_address: Vec<u8>,
    pub token_id: i64,
    pub amount: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = crate::orm::schema::processed_block)]
pub struct ProcessedBlock {
    pub id: i32,
    pub height: i32,
    pub processed_at: chrono::NaiveDateTime,
    pub success: bool,
}