mod log;

use std::sync::Arc;

use crate::{
    context::ChainConfig,
    repository::OrderRepository,
    solidity::Orderbook::{self},
};
use alloy::sol_types::SolEvent;
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockTransactionsKind, Filter},
};
use eyre::{Ok, Result};
use futures_util::StreamExt;
use log::WorkerLog;
use tracing::{debug, info, warn};
pub(crate) struct Worker {
    chain_config: ChainConfig,
    order_repository: Arc<OrderRepository>,
}

impl Worker {
    pub async fn new(postgres_url: String, chain_config: ChainConfig) -> Result<Self> {
        let order_repository = Arc::new(OrderRepository::new(postgres_url).await?);
        Ok(Worker {
            chain_config,
            order_repository,
        })
    }

    pub async fn run_block_listener_poll(&self) -> Result<()> {
        let url = &self.chain_config.rpc_url;
        let address: Address = self.chain_config.order_contract_address.parse()?;
        // TODO Take it from config only if it doesn't exist in the DB
        let starting_height = self.chain_config.start_block.unwrap_or(0);
        debug!(
            url, %address, from_block = starting_height,
            "Poll worker routine is starting!"
        );

        let provider = ProviderBuilder::new().on_builtin(url).await?;

        // TODO decide whether to work with Safe or Finalized or Latest block
        let latest_block = provider
            .get_block_by_number(
                alloy::eips::BlockNumberOrTag::Latest,
                BlockTransactionsKind::Full,
            )
            .await?;
        let latest_height = match latest_block {
            Some(block) => block.header.number,
            None => {
                warn!("No latest block number found!");
                starting_height
            }
        };

        for block_number in starting_height..=latest_height {
            debug!("Processing block number {block_number}...");

            let filter = Filter::new()
                .address(address)
                .events([
                    Orderbook::OrderCreated::SIGNATURE,
                    Orderbook::OrderWithdrawn::SIGNATURE,
                    Orderbook::OrderFilled::SIGNATURE,
                ])
                .from_block(block_number)
                .to_block(block_number);

            let sub = provider.get_logs(&filter).await?;
            for log in sub {
                let worker_log = WorkerLog::new(Arc::clone(&self.order_repository)).await?;
                worker_log.process_event_log(log).await?;
            }

            debug!("Block number {block_number} correctly processed!");
        }

        Ok(())
    }

    pub async fn run_block_listener_subscription(&self) -> Result<()> {
        let url = &self.chain_config.ws_url;
        let address: Address = self.chain_config.order_contract_address.parse()?;
        let start_block_height = self.chain_config.start_block.unwrap_or(0);

        info!(
            url, %address, start_block_height,
            "Subscription worker routine is starting!"
        );

        let provider = ProviderBuilder::new().on_builtin(url).await?;

        let sub = provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        while let Some(_) = stream.next().await {
            self.run_block_listener_poll().await?;
        }

        info!("Subscription routine terminated!");
        Ok(())
    }
}
