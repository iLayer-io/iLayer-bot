use crate::{
    context::AppContext,
    dao::{self, OrderDao},
    solidity::Orderbook::{OrderCreated, OrderFilled, OrderWithdrawn},
};
use alloy::primitives::Log;
use diesel::PgConnection;
use eyre::{Ok, Result};

pub async fn process_order_withdrawn_log(
    _context: &AppContext,
    connection: PgConnection,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    let mut user_impl = dao::UserImpl { conn: connection };

    let _result = user_impl.get_order(log.data.orderId.to_vec());
    return Ok(());
}

pub async fn process_order_filled_log(
    _context: &AppContext,
    _connection: PgConnection,
    _log: Log<OrderFilled>,
) -> Result<()> {
    Ok(())
}

pub async fn process_order_created_log(
    _context: &AppContext,
    _connection: PgConnection,
    _log: Log<OrderCreated>,
) -> Result<()> {
    Ok(())
}
