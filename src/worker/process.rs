use crate::{
    context::AppContext,
    dao::{self, OrderDao},
    solidity::{map_solidity_order_to_model, Orderbook::{OrderCreated, OrderFilled, OrderWithdrawn}},
};
use alloy::primitives::Log;
use diesel::PgConnection;
use eyre::{Ok, Result};

pub async fn process_order_withdrawn_log(
    _context: &AppContext,
    _connection: PgConnection,
    _log: Log<OrderWithdrawn>,
) -> Result<()> {
    // TODO Check for order existence and set it as withdrawn
    return Ok(());
}

pub async fn process_order_filled_log(
    _context: &AppContext,
    _connection: PgConnection,
    _log: Log<OrderFilled>,
) -> Result<()> {
    // TODO Check for order existence and set it as filled
    Ok(())
}

pub async fn process_order_created_log(
    _context: &AppContext,
    connection: PgConnection,
    log: Log<OrderCreated>,
) -> Result<()> {
    // TODO Check for order existence and skip if it already exists
    let mut user_impl = dao::UserImpl { conn: connection };
    let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;
    let _result = user_impl.create_order(new_order)?;
    Ok(())
}
