use crate::context::AppContext;
use alloy::rpc::types::Log;
use eyre::{Ok, Result};

pub async fn process_order_withdrawn_log(context: &AppContext, log: Log) -> Result<()> {
    Ok(())
}

pub async fn process_order_filled_log(context: &AppContext, log: Log) -> Result<()> {
    Ok(())
}

pub async fn process_order_created_log(context: &AppContext, log: Log) -> Result<()> {
    Ok(())
}