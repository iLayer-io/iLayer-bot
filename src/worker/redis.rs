use eyre::Result;

use crate::{context::AppContext, dao::models::Order};

pub async fn run_order_filler_worker(context: &AppContext) -> Result<()> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(context.config.redis_poll_interval)).await;
        // TODO:
        // 1. Get all ready orders from Redis
        // 2. Try to Fill the Orders
        //   - call fillOrder on the target smart contract's router
        // 3. If successful, mark as done the Redis order
    }
}


pub async fn filter_orders<'a>(context: &AppContext, orders: &'a [Order]) -> Result<&'a [Order]> {
    Ok(orders)
}