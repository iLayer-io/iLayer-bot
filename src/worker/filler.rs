use eyre::Result;
use slog::info;

use crate::{context::AppContext, dao::sql::new};

pub async fn run_order_filler_worker(context: &AppContext) -> Result<()> {
    let mut order_dao = new(context).await?;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(context.config.redis_poll_interval)).await;
        let ready_orders = order_dao.get_ready_orders().await?;
        for order in ready_orders {
            info!(context.logger, "Trying to fill ready order with order_id: {:?}", hex::encode(order.order_id));
            // TODO:
            // 2. Try to Fill the Orders
            //   - call fillOrder on the target smart contract's router
            // 3. If successful, mark as done the Redis order
        }
    }
}