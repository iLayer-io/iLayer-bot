use std::sync::Arc;

use eyre::Result;
use futures_util::StreamExt;
use tracing::info;

use crate::{
    client::redis::CHANNEL, context::ChainConfig, repository::order::OrderRepository,
    service::Service,
};

pub(crate) struct Filler {
    chain_config: ChainConfig,
    order_repository: Arc<OrderRepository>,
    redis_client: redis::Client,
}

impl Filler {
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

impl Service for Filler {
    async fn _run(&self) -> Result<()> {
        let mut pubsub = self.redis_client.get_async_pubsub().await?;
        let channel = format!("{}:{}", CHANNEL, self.chain_config.chain_id);
        pubsub.subscribe(channel.clone()).await?;

        info!(channel = channel.clone(), "Filler has started listening");
        let mut stream = pubsub.on_message();
        loop {
            let (msg, new_stream) = stream.into_future().await;
            stream = new_stream;

            match msg {
                Some(msg) => {
                    let payload: String = msg.get_payload()?;
                    let order: entity::order::Model = serde_json::from_str(&payload)?;
                    info!(
                        order_id = hex::encode(order.order_id.clone()),
                        "Trying to fill ready order",
                    );

                    self.order_repository
                        .update_order_status(
                            order.order_id.clone(),
                            entity::sea_orm_active_enums::OrderStatus::Filled,
                        )
                        .await?;

                    info!(order_id = hex::encode(order.order_id), "Order filled",);
                }
                None => {}
            }
        }
    }

    fn service_name(&self) -> String {
        format!("{} Filler", self.chain_config.name)
    }
}
