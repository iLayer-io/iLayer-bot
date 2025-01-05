use crate::{
    context::AppContext,
    repository::sql::new,
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
use slog::{debug, info, trace, warn};

use crate::solidity::Orderbook;

pub async fn process_order_withdrawn_log(
    context: &AppContext,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    info!(context.logger, "Processing Order Withdrawn event..."; "log" => format!("{:?}", log.orderId));

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(context.logger, "Order Withdrawn event processed successfully!"; "log" => format!("{:?}", log.orderId));
    return Ok(());
}

pub async fn process_order_filled_log(context: &AppContext, log: Log<OrderFilled>) -> Result<()> {
    info!(context.logger, "Processing Order Filled event..."; "log" => format!("{:?}", log.orderId));

    let user_impl = new(context).await?;
    user_impl.delete_order(log.orderId.to_vec()).await?;

    info!(context.logger, "Order Filled event processed successfully!"; "log" => format!("{:?}", log.orderId));
    Ok(())
}

pub async fn process_order_created_log(context: &AppContext, log: Log<OrderCreated>) -> Result<()> {
    info!(context.logger, "Processing Order Created event..."; "log" => format!("{:?}", log.orderId));

    let user_impl = new(context).await?;

    let order_exists = user_impl.get_order(log.orderId.to_vec()).await.is_ok();
    if order_exists {
        info!(context.logger, "Order already exists, skipping..."; "log" => format!("{:?}", log.orderId));
        return Ok(());
    }

    let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;
    user_impl.create_order(&new_order).await?;
    info!(context.logger, "Order Created event processed successfully!"; "log" => format!("{:?}", log.orderId));
    Ok(())
}

pub async fn process_event_log(context: &AppContext, log: alloy::rpc::types::Log) -> Result<()> {
    trace!(context.logger, "Processing Event Log"; "log" => format!("{:?}", log.data()));
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

    warn!(context.logger, "Unable to decode log"; "log" => format!("{:?}", log));
    Err(eyre::eyre!("Unable to decode log"))
}

pub async fn run_block_listener_poll_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.rpc_url;
    let address: Address = context.config.order_contract_address.parse()?;
    // TODO Take it from config only if it doesn't exist in the DB
    let starting_height = context.config.from_block.unwrap_or(0);
    let block_confirmations = context.config.block_confirmations;
    debug!(context.logger, "Poll worker routine is starting!"; 
        "url" => url, 
        "address" => format!("{:?}", address), 
        "from_block" => starting_height, 
        "block_confirmations" => block_confirmations);

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
            warn!(context.logger, "No latest block number found!");
            starting_height
        }
    };

    if starting_height == latest_height {
        return Ok(());
    }

    for block_number in starting_height..=latest_height {
        debug!(
            context.logger,
            "Processing block number {}...", block_number
        );

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

        debug!(
            context.logger,
            "Block number {} correctly processed!", block_number
        );
    }

    Ok(())
}

pub async fn run_block_listener_subscription_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.ws_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);

    info!(context.logger, "Subscription worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let sub = provider.subscribe_blocks().await?;
    let mut stream = sub.into_stream();

    while let Some(_) = stream.next().await {
        run_block_listener_poll_worker(context).await?;
    }

    info!(context.logger, "Subscription routine terminated!");
    Ok(())
}
// // TODO NB. Subscription doesn't support the concept of Safe and Finalized blocks. it may not be suitable for us
// //   NB. It start arising the need to use a custom blockchain node to be notified about new blocks.
// pub async fn run_event_listener_subscription_worker(context: &AppContext) -> Result<()> {
//     // TODO FIXME: Refactor to subscribe to blocks and save processed blocks into DB
//     //   or use a more efficient way to process logs,
//     //   but we need to keep track of the last processed ones and avoid duplicates/out of order processing
//     let url = &context.config.ws_url;
//     let address: Address = context.config.order_contract_address.parse()?;
//     let from_block = context.config.from_block.unwrap_or(0);

//     info!(context.logger, "Subscription worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block);

//     let provider = ProviderBuilder::new().on_builtin(url).await?;

//     let filter = Filter::new()
//         .address(address)
//         .events([
//             Orderbook::OrderCreated::SIGNATURE,
//             Orderbook::OrderWithdrawn::SIGNATURE,
//             Orderbook::OrderFilled::SIGNATURE,
//         ])
//         .from_block(from_block);

//     // TODO FIXME subscribe for blocks iof logs
//     let sub = provider.subscribe_logs(&filter).await?;
//     let mut stream = sub.into_stream();

//     while let Some(log) = stream.next().await {
//         process_event_log(context, log).await?;
//     }

//     info!(context.logger, "Subscription routine terminated!");
//     Ok(())
// }
