pub mod filler;

use crate::{
    context::AppContext,
    repository::new,
    solidity::{
        map_solidity_order_to_model,
        Orderbook::{self, OrderCreated, OrderFilled, OrderWithdrawn},
    },
};
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockTransactionsKind, Filter},
};
use alloy::{primitives::Log, sol_types::SolEvent};
use eyre::{Ok, Result};
use futures_util::StreamExt;
use tracing::{debug, info, trace, warn};

pub async fn process_order_withdrawn_log(
    context: &AppContext,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    info!(
        order_id = hex::encode(log.orderId),
        "Processing Order Withdrawn event..."
    );

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(
        order_id = hex::encode(log.orderId),
        "Order Withdrawn event processed successfully!"
    );
    Ok(())
}

pub async fn process_order_filled_log(context: &AppContext, log: Log<OrderFilled>) -> Result<()> {
    info!(
        order_id = hex::encode(log.orderId),
        "Processing Order Filled event..."
    );

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(
        order_id = hex::encode(log.orderId),
        "Order Filled event processed successfully!"
    );
    Ok(())
}

pub async fn process_order_created_log(context: &AppContext, log: Log<OrderCreated>) -> Result<()> {
    info!(
        order_id = hex::encode(log.orderId),
        "Processing Order Created event..."
    );

    let user_impl = new(context).await?;

    let order_exists = user_impl.get_order(log.orderId.to_vec()).await.is_ok();
    if order_exists {
        info!(
            order_id = hex::encode(log.orderId),
            "Order already exists, skipping..."
        );
    } else {
        let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;
        user_impl.create_order(&new_order).await?;
        info!(
            order_id = hex::encode(log.orderId),
            "Order Created event processed successfully!"
        );
    }
    Ok(())
}

pub async fn process_event_log(context: &AppContext, log: alloy::rpc::types::Log) -> Result<()> {
    trace!(log_data = ?log.data(), "Processing Event Log");
    let order_created = Orderbook::OrderCreated::decode_log(&log.inner, false);
    if order_created.is_ok() {
        let order_created = order_created.unwrap();
        process_order_created_log(context, order_created.clone()).await?;
        return Ok(());
    }

    let order_filled = Orderbook::OrderFilled::decode_log(&log.inner, false);
    if order_filled.is_ok() {
        let order_filled = order_filled.unwrap();
        process_order_filled_log(context, order_filled.clone()).await?;
        return Ok(());
    }

    let order_withdrawn = Orderbook::OrderWithdrawn::decode_log(&log.inner, false);
    if order_withdrawn.is_ok() {
        let order_withdrawn = order_withdrawn.unwrap();
        process_order_withdrawn_log(context, order_withdrawn.clone()).await?;
        return Ok(());
    }

    warn!(log = ?log, "Unable to decode log");
    Err(eyre::eyre!("Unable to decode log"))
}

pub async fn run_block_listener_poll_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.rpc_url;
    let address: Address = context.config.order_contract_address.parse()?;
    // TODO Take it from config only if it doesn't exist in the DB
    let starting_height = context.config.from_block.unwrap_or(0);
    let block_confirmations = context.config.block_confirmations;
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
        Some(block) => block.header.number - block_confirmations,
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
            process_event_log(context, log).await?;
        }

        debug!("Block number {block_number} correctly processed!");
    }

    Ok(())
}

pub async fn run_block_listener_subscription_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.ws_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);

    info!(
        url, %address, from_block,
        "Subscription worker routine is starting!"
    );

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let sub = provider.subscribe_blocks().await?;
    let mut stream = sub.into_stream();

    while let Some(_) = stream.next().await {
        run_block_listener_poll_worker(context).await?;
    }

    info!("Subscription routine terminated!");
    Ok(())
}
