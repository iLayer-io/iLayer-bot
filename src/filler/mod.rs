use std::sync::Arc;

use eyre::Result;
use tracing::{error, info};

use crate::{context::ChainConfig, repository::OrderRepository};

pub(crate) struct Filler {
    chain_config: ChainConfig,
    order_repository: Arc<OrderRepository>,
}

impl Filler {
    pub async fn new(postgres_url: String, chain_config: ChainConfig) -> Result<Self> {
        let order_repository = Arc::new(OrderRepository::new(postgres_url).await?);
        Ok(Self {
            chain_config,
            order_repository,
        })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            match self._run().await {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        error = %e,
                        chain_id = self.chain_config.chain_id,
                        "Error in filler service");
                }
            }

            // TODO Maybe we should make this configurable?
            tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        }
    }

    async fn _run(&self) -> Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.chain_config.filler_poll_interval,
            ))
            .await;

            // TODO Filter by chain id
            let ready_orders = self.order_repository.get_ready_orders().await?;
            for order in ready_orders {
                info!(
                    order_id = hex::encode(order.order_id),
                    "Trying to fill ready order",
                );
                // TODO:
                // 2. Try to Fill the Orders
                //   - call fillOrder on the target smart contract's router
                // 3. If successful, mark as done the order
            }
        }
    }
}
