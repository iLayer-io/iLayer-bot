use serde::{Deserialize, Serialize};

#[derive(Default,Debug, Serialize,Deserialize)]
pub struct Order {
    pub user: Vec<u8>,
    pub id: Vec<u8>,
    pub filler: Vec<u8>,
    pub source_chain_selector: Vec<u8>,
    pub destination_chain_selector: Vec<u8>,
    pub sponsored: bool,
    pub primary_filler_deadline: chrono::DateTime<chrono::Utc>,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub call_recipient: Vec<u8>,
    pub call_data: Vec<u8>,
}