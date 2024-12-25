use crate::{context::AppContext, models::Order, schema::orders::dsl::*};
use alloy::rpc::types::Log;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use eyre::{Ok, Result};

pub async fn process_order_withdrawn_log(context: &mut AppContext, log: Log) -> Result<()> {
    let results = orders
        .limit(5)
        .select(Order::as_select())
        .load(&mut context.connection)
        .expect("Error loading posts");
    Ok(())
}

pub async fn process_order_filled_log(context: &AppContext, log: Log) -> Result<()> {
    Ok(())
}

pub async fn process_order_created_log(context: &AppContext, log: Log) -> Result<()> {
    Ok(())
}