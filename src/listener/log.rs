use bot_solidity::{
    map_solidity_order_to_model,
    OrderHub::{self, OrderCreated, OrderFilled, OrderWithdrawn},
};

use alloy::{primitives::Log, sol_types::SolEvent};
use entity::sea_orm_active_enums::OrderStatus;
use eyre::Result;
use sea_orm::TryIntoModel;
use tracing::{info, trace, warn};

impl super::Listener {
    async fn process_order_withdrawn_log(&self, log: Log<OrderWithdrawn>) -> Result<()> {
        info!(
            order_id = hex::encode(log.orderId),
            "Processing Order Withdrawn event"
        );

        self.order_repository
            .update_order_status(log.orderId.to_vec(), OrderStatus::Withdrawn)
            .await?;

        info!(
            order_id = hex::encode(log.orderId),
            "Order Withdrawn event processed successfully!"
        );
        Ok(())
    }

    async fn process_order_filled_log(&self, log: Log<OrderFilled>) -> Result<()> {
        info!(
            order_id = hex::encode(log.orderId),
            "Processing Order Filled event"
        );

        self.order_repository
            .update_order_status(log.orderId.to_vec(), OrderStatus::Filled)
            .await?;

        info!(
            order_id = hex::encode(log.orderId),
            "Order Filled event processed successfully!"
        );
        Ok(())
    }

    async fn process_order_created_log(
        &self,
        log: Log<OrderCreated>,
        should_publish_to_redis: bool,
    ) -> Result<()> {
        info!(
            order_id = hex::encode(log.orderId),
            "Processing Order Created event"
        );

        let order_exists = self
            .order_repository
            .get_order(log.orderId.to_vec())
            .await
            .is_ok();
        if order_exists {
            info!(
                order_id = hex::encode(log.orderId),
                "Order already exists, skipping"
            );
            return Ok(());
        }

        let mut new_order = map_solidity_order_to_model(
            self.chain_config.chain_id,
            log.orderId.to_vec(),
            &log.order,
        )?;

        new_order.id =
            sea_orm::ActiveValue::Set(self.order_repository.create_order(&new_order).await?);

        if should_publish_to_redis {
            crate::client::redis::publish(
                &mut self.redis_client.get_multiplexed_async_connection().await?,
                new_order.clone().try_into_model()?,
                self.chain_config.chain_id,
            )
            .await?;
        }

        info!(
            order_id = hex::encode(log.orderId),
            "Order Created event processed successfully!"
        );
        Ok(())
    }

    pub async fn process_event_log(
        &self,
        log: &alloy::rpc::types::Log,
        should_publish_to_redis: bool,
    ) -> Result<()> {
        // NB. this process_event_log function is called from both the run_subscription and run_polling functions.
        // this function will be called also when processing logs "in batch" at the start of a "blockchain reindexing" process.
        // we need to be careful to write it with idempotency in mind.

        trace!(log_data = ?log.data(), "Processing Event Log");
        let order_created = OrderHub::OrderCreated::decode_log(&log.inner, false);
        if order_created.is_ok() {
            let order_created = order_created.unwrap();
            self.process_order_created_log(order_created.clone(), should_publish_to_redis)
                .await?;
            return Ok(());
        }

        let order_filled = OrderHub::OrderFilled::decode_log(&log.inner, false);
        if order_filled.is_ok() {
            let order_filled = order_filled.unwrap();
            self.process_order_filled_log(order_filled.clone()).await?;
            return Ok(());
        }

        let order_withdrawn = OrderHub::OrderWithdrawn::decode_log(&log.inner, false);
        if order_withdrawn.is_ok() {
            let order_withdrawn = order_withdrawn.unwrap();
            self.process_order_withdrawn_log(order_withdrawn.clone())
                .await?;
            return Ok(());
        }

        warn!(log = ?log, "Unable to decode log");
        Err(eyre::eyre!("Unable to decode log"))
    }
}
