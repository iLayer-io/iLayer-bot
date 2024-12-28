use crate::{
    context::{context, AppContext},
    dao::{self, redis::OrderDao},
    solidity::{map_solidity_order_to_model, Orderbook::{OrderCreated, OrderFilled, OrderWithdrawn}},
};
use alloy::primitives::Log;
use eyre::{Ok, Result};
use redis::Connection;
use slog::info;

pub async fn process_order_withdrawn_log(
    context: &AppContext,
    connection: Connection,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    let mut user_impl = dao::redis::UserImpl::new(connection, context);
    user_impl.delete_order(log.orderId.to_vec()).await?;
    return Ok(());
}

pub async fn process_order_filled_log(
    context: &AppContext,
    connection: Connection,
    log: Log<OrderFilled>,
) -> Result<()> {
    // TODO Check for order existence and set it as filled
    let mut user_impl = dao::redis::UserImpl::new(connection, context);
    user_impl.delete_order(log.orderId.to_vec()).await?;
    Ok(())
}

pub async fn process_order_created_log(
    context: &AppContext,
    connection: Connection,
    log: Log<OrderCreated>,
) -> Result<()> {
    // TODO Check for order existence and skip if it already exists
    info!(context.logger, "Processing log..."; "log" => format!("{:?}", log));
    let mut user_impl = dao::redis::UserImpl::new(connection, context);

    info!(context.logger, "map solidity to model..."; "log" => format!("{:?}", log));
    let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;

    info!(context.logger, "creating order..."; "log" => format!("{:?}", log));
    let _result = user_impl.create_order(new_order).await?;
    info!(context.logger, "Processed log!"; "log" => format!("{:?}", log));
    
    Ok(())
}
