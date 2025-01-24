use std::sync::Arc;

use eyre::Result;

use tracing::debug;

use crate::{context::ChainConfig, repository::order::OrderRepository, service::Service};

pub(crate) struct Watcher {
    chain_config: ChainConfig,
    order_repository: Arc<OrderRepository>,
    redis_client: redis::Client,
}

impl Watcher {
    pub async fn new(
        postgres_url: String,
        redis_url: String,
        chain_config: ChainConfig,
    ) -> Result<Self> {
        let order_repository = Arc::new(OrderRepository::new(postgres_url).await?);
        let redis_client = redis::Client::open(redis_url)?;
        Ok(Self {
            chain_config,
            order_repository,
            redis_client,
        })
    }
}

impl Service for Watcher {
    async fn _run(&self) -> Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                self.chain_config.filler_poll_interval,
            ))
            .await;

            let ready_orders = self
                .order_repository
                .get_ready_orders(self.chain_config.chain_id)
                .await?;

            for order in ready_orders {
                let order_id = order.order_id.clone();
                debug!(
                    order_id = hex::encode(order_id),
                    "Publish on redis the ready order",
                );

                crate::client::redis::publish(
                    &mut self.redis_client.get_multiplexed_async_connection().await?,
                    order,
                    self.chain_config.chain_id,
                )
                .await?;
            }
        }
    }

    fn service_name(&self) -> String {
        format!("Watcher")
    }
}
