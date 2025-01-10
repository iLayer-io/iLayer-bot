pub mod filler;

use crate::{
    context::AppContext,
    repository::new,
    solidity::{
        map_solidity_order_to_model,
        Orderbook::{OrderCreated, OrderFilled, OrderWithdrawn},
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

use crate::solidity::Orderbook;

pub async fn process_order_withdrawn_log(
    context: &AppContext,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    info!(
        message = "Processing Order Withdrawn event...",
        order.order_id = format!("{:?}", log.orderId)
    );

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(
        message = "Order Withdrawn event processed successfully!",
        order.order_id = format!("{:?}", log.orderId)
    );
    return Ok(());
}

pub async fn process_order_filled_log(context: &AppContext, log: Log<OrderFilled>) -> Result<()> {
    info!(
        message = "Processing Order Filled event...",
        order.order_id = format!("{:?}", log.orderId)
    );

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(
        message = "Order Filled event processed successfully!",
        order.order_id = format!("{:?}", log.orderId)
    );
    Ok(())
}

pub async fn process_order_created_log(context: &AppContext, log: Log<OrderCreated>) -> Result<()> {
    info!(
        message = "Processing Order Created event...",
        order.order_id = format!("{:?}", log.orderId)
    );

    let user_impl = new(context).await?;

    let order_exists = user_impl.get_order(log.orderId.to_vec()).await.is_ok();
    if order_exists {
        info!(
            message = "Order already exists, skipping...",
            order.order_id = format!("{:?}", log.orderId)
        );
        return Ok(());
    }

    let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;
    user_impl.create_order(&new_order).await?;
    info!(
        message = "Order Created event processed successfully!",
        order.order_id = format!("{:?}", log.orderId)
    );
    Ok(())
}

pub async fn process_event_log(context: &AppContext, log: alloy::rpc::types::Log) -> Result<()> {
    trace!(
        message = "Processing Event Log",
        log = format!("{:?}", log.data())
    );
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

    warn!(
        message = "Unable to decode log",
        "log" = format!("{:?}", log)
    );
    Err(eyre::eyre!("Unable to decode log"))
}

pub async fn run_block_listener_poll_worker(context: &AppContext) -> Result<()> {
    let url = "http://127.0.0.1:8545";
    let address: Address = "0x8ce361602B935680E8DeC218b820ff5056BeB7af".parse()?;
    // TODO Take it from config only if it doesn't exist in the DB
    let starting_height = 0;
    debug!(
        message = "Poll worker routine is starting!",
        url = url,
        address = format!("{:?}", address),
        from_block = starting_height,
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

    if starting_height == latest_height {
        return Ok(());
    }

    for block_number in starting_height..=latest_height {
        debug!(message = "Processing block number.", block_number);

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

        debug!(message = "Block correctly processed!", block_number);
    }

    Ok(())
}

pub async fn run_block_listener_subscription_worker(context: &AppContext) -> Result<()> {
    let url = "ws://127.0.0.1:8545";
    let address: Address = "0x8ce361602B935680E8DeC218b820ff5056BeB7af".parse()?;
    let from_block = 0;

    info!(message = "Subscription worker routine is starting!", "url" = url, "address" = address.to_string(), "from_block" = from_block);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let sub = provider.subscribe_blocks().await?;
    let mut stream = sub.into_stream();

    while let Some(_) = stream.next().await {
        run_block_listener_poll_worker(context).await?;
    }

    info!("Subscription routine terminated!");
    Ok(())
}
