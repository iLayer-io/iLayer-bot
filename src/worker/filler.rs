use eyre::Result;
use slog::info;

use crate::{context::AppContext, repository::new};

pub async fn run_order_filler_worker(context: &AppContext) -> Result<()> {
    let order_repository = new(context).await?;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(context.config.filler_poll_interval)).await;
        let ready_orders = order_repository.get_ready_orders().await?;
        for order in ready_orders {
            info!(context.logger, "Trying to fill ready order with order_id: {:?}", hex::encode(order.order_id));
            // TODO:
            // 2. Try to Fill the Orders
            //   - call fillOrder on the target smart contract's router
            // 3. If successful, mark as done the order
        }
    }
}