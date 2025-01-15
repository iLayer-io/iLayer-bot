mod log;

use std::sync::Arc;

use crate::{
    context::ChainConfig,
    repository::{block_checkpoint::BlockCheckpointRepository, order::OrderRepository},
    service::Service,
    solidity::Orderbook::{self},
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockTransactionsKind, Filter},
    sol_types::SolEvent,
};
use eyre::Result;
use tracing::{debug, info, warn};

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

    async fn run_polling(&self) -> Result<()> {
        let url = &self.chain_config.rpc_url;
        let address: Address = self.chain_config.order_contract_address.parse()?;
        let block_batch_size = self.chain_config.block_batch_size.unwrap_or(1_000);

        // Take the starting block height from the database, check if it is coherent with the configured starting block
        let config_starting_block = self.chain_config.start_block;
        let db_starting_block = self
            .block_checkpoint_repository
            .get_last_block_checkpoint()
            .await?
            .map(|checkpoint| (checkpoint.height as u64) + 1);

        let mut from_block = match (config_starting_block, db_starting_block) {
            (Some(config_starting_block), Some(db_starting_block)) => {
                if db_starting_block < config_starting_block {
                    return Err(eyre::eyre!(
                        "Starting block from the database is less than the configured starting block. \
                        Orders may remain stuck forever. Please fix the database inconsistency."
                    ));
                }

                db_starting_block
            }
            (None, Some(db_starting_block)) => db_starting_block,
            (Some(config_starting_block), None) => config_starting_block,
            (None, None) => {
                return Ok(());
            }
        };

        let provider = ProviderBuilder::new().on_builtin(url).await?;

        // TODO decide whether to work with Safe or Finalized or Latest block
        let latest_block = provider
            .get_block_by_number(
                alloy::eips::BlockNumberOrTag::Latest,
                BlockTransactionsKind::Full,
            )
            .await?
            .expect("Latest block must exist!")
            .header
            .number;

        debug!(
            url, %address, from_block, latest_block,
            "Polling routine starting!"
        );

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
                self.process_event_log(&log).await?;
            }

            debug!(from_block, to_block, latest_block, "Block batch processed!");

            self.block_checkpoint_repository
                .create_block_checkpoint(self.chain_config.chain_id, to_block)
                .await?;
            from_block = to_block + 1;
        }

        Ok(())
    }

    async fn run_subscription(&self) -> Result<()> {
        let url = &self.chain_config.ws_url;
        let address: Address = self.chain_config.order_contract_address.parse()?;

        let starting_block = self
            .block_checkpoint_repository
            .get_last_block_checkpoint()
            .await?
            .map(|checkpoint| BlockNumberOrTag::Number(checkpoint.height as u64 + 1))
            .unwrap_or(BlockNumberOrTag::Finalized); // TODO: define if this should be Finalized or Latest

        info!(
            url, %address, ?starting_block,
            "Subscription worker routine is starting!"
        );

        let filter = Filter::new()
            .address(address)
            .events([
                Orderbook::OrderCreated::SIGNATURE,
                Orderbook::OrderWithdrawn::SIGNATURE,
                Orderbook::OrderFilled::SIGNATURE,
            ])
            .from_block(starting_block);

        let provider = ProviderBuilder::new().on_builtin(url).await?;
        let mut sub = provider.subscribe_logs(&filter).await?;

        loop {
            let log = sub.recv().await?;
            self.process_event_log(&log).await?;
            match log.block_number {
                Some(n) => {
                    self.block_checkpoint_repository
                        .create_block_checkpoint(self.chain_config.chain_id, n)
                        .await?
                }
                None => warn!(?log, "No block number found for log"),
            }
        }
    }
}

impl Service for Listener {
    async fn _run(&self) -> Result<()> {
        self.run_polling().await?;
        self.run_subscription().await
    }

    fn service_name(&self) -> String {
        format!("{} Listener", self.chain_config.name)
    }
}
