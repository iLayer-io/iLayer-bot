mod log;

use std::sync::Arc;

use crate::{
    context::ChainConfig,
    repository::{block_checkpoint::BlockCheckpointRepository, order::OrderRepository},
    solidity::Orderbook::{self},
};
use alloy::sol_types::SolEvent;
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockTransactionsKind, Filter},
};
use eyre::Result;
use futures_util::StreamExt;
use log::WorkerLog;
use tracing::{debug, error, info, warn};

pub(crate) struct Listener {
    chain_config: ChainConfig,
    order_repository: Arc<OrderRepository>,
    block_checkpoint_repository: Arc<BlockCheckpointRepository>,
}

impl Listener {
    pub async fn new(postgres_url: String, chain_config: ChainConfig) -> Result<Self> {
        let order_repository = Arc::new(OrderRepository::new(postgres_url.clone()).await?);
        let block_checkpoint_repository =
            Arc::new(BlockCheckpointRepository::new(postgres_url.clone()).await?);
        Ok(Self {
            chain_config,
            order_repository,
            block_checkpoint_repository,
        })
    }

    pub async fn run_subscription(&self) -> Result<()> {
        loop {
            match self._run_subscription().await {
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

    pub async fn run_polling(&self) -> Result<()> {
        let url = &self.chain_config.rpc_url;
        let address: Address = self.chain_config.order_contract_address.parse()?;
        let block_batch_size = self.chain_config.block_batch_size.unwrap_or(1_000);

        // Take the starting block height from the database, check if it is coherent with the configured starting block
        let config_starting_block = self.chain_config.start_block.unwrap_or(0);
        let starting_block = self
            .block_checkpoint_repository
            .get_last_block_checkpoint()
            .await
            .map_or(config_starting_block, |checkpoint| {
                (checkpoint.height as u64) + 1
            });

        if starting_block > config_starting_block {
            warn!(
                starting_block,
                config_starting_block, "Taking the starting block height from the database"
            );
        }
        if starting_block < config_starting_block {
            return Err(eyre::eyre!(
                "Starting block from the database is less than the configured starting block. \
                Orders may remain stuck forever. Please fix the database inconsistency."
            ));
        }

        let provider = ProviderBuilder::new().on_builtin(url).await?;

        // TODO decide whether to work with Safe or Finalized or Latest block
        let latest_block = provider
            .get_block_by_number(
                alloy::eips::BlockNumberOrTag::Latest,
                BlockTransactionsKind::Full,
            )
            .await?;

        let latest_block = match latest_block {
            Some(block) => block.header.number,
            None => {
                return Err(eyre::eyre!("No latest block number found"));
            }
        };

        debug!(
            url, %address, config_starting_block, latest_block,
            "Polling routine starting!"
        );

        let mut from_block = config_starting_block;
        while from_block <= latest_block {
            let to_block = block_batch_size + from_block;
            let to_block = std::cmp::min(to_block, latest_block);
            debug!(from_block, to_block, "Processing block batch");

            let filter = Filter::new()
                .address(address)
                .events([
                    Orderbook::OrderCreated::SIGNATURE,
                    Orderbook::OrderWithdrawn::SIGNATURE,
                    Orderbook::OrderFilled::SIGNATURE,
                ])
                .from_block(from_block)
                .to_block(to_block);

            // TODO index add chain_id column to the order table
            // TODO should we use a db tx?
            let sub = provider.get_logs(&filter).await?;
            for log in sub {
                let worker_log = WorkerLog::new(
                    Arc::clone(&self.order_repository),
                    self.chain_config.chain_id,
                )
                .await?;
                worker_log.process_event_log(log).await?;
            }

            debug!(from_block, to_block, latest_block, "Block batch processed!");

            self.block_checkpoint_repository
                .create_block_checkpoint(self.chain_config.chain_id, to_block)
                .await?;
            from_block = to_block + 1;
        }

        Ok(())
    }

    async fn _run_subscription(&self) -> Result<()> {
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

        while (stream.next().await).is_some() {
            self.run_polling().await?;
        }

        info!("Subscription routine terminated!");
        Ok(())
    }
}
