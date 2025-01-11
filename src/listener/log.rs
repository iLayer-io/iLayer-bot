use std::sync::Arc;

use crate::{
    repository::order::OrderRepository,
    solidity::{
        map_solidity_order_to_model,
        Orderbook::{self, OrderCreated, OrderFilled, OrderWithdrawn},
    },
};
use alloy::{primitives::Log, sol_types::SolEvent};
use eyre::{Ok, Result};
use tracing::{info, trace, warn};

pub struct WorkerLog {
    // TODO Maybe find me a better name
    order_repository: Arc<OrderRepository>,
}

impl WorkerLog {
    pub async fn new(order_repository: Arc<OrderRepository>) -> Result<Self> {
        Ok(WorkerLog { order_repository })
    }

    pub async fn process_order_withdrawn_log(&self, log: Log<OrderWithdrawn>) -> Result<()> {
        info!(
            order_id = hex::encode(log.orderId),
            "Processing Order Withdrawn event"
        );

        self.order_repository
            .delete_order(log.orderId.to_vec())
            .await?;

        info!(
            order_id = hex::encode(log.orderId),
            "Order Withdrawn event processed successfully!"
        );
        Ok(())
    }

    pub async fn process_order_filled_log(&self, log: Log<OrderFilled>) -> Result<()> {
        info!(
            order_id = hex::encode(log.orderId),
            "Processing Order Filled event"
        );

        self.order_repository
            .delete_order(log.orderId.to_vec())
            .await?;

        info!(
            order_id = hex::encode(log.orderId),
            "Order Filled event processed successfully!"
        );
        Ok(())
    }

    pub async fn process_order_created_log(&self, log: Log<OrderCreated>) -> Result<()> {
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
        } else {
            let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;
            self.order_repository.create_order(&new_order).await?;
            info!(
                order_id = hex::encode(log.orderId),
                "Order Created event processed successfully!"
            );
        }
        Ok(())
    }

    pub async fn process_event_log(&self, log: alloy::rpc::types::Log) -> Result<()> {
        trace!(log_data = ?log.data(), "Processing Event Log");
        let order_created = Orderbook::OrderCreated::decode_log(&log.inner, false);
        if order_created.is_ok() {
            let order_created = order_created.unwrap();
            self.process_order_created_log(order_created.clone())
                .await?;
            return Ok(());
        }

        let order_filled = Orderbook::OrderFilled::decode_log(&log.inner, false);
        if order_filled.is_ok() {
            let order_filled = order_filled.unwrap();
            self.process_order_filled_log(order_filled.clone()).await?;
            return Ok(());
        }

        let order_withdrawn = Orderbook::OrderWithdrawn::decode_log(&log.inner, false);
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
