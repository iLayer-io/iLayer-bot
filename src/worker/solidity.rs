use crate::{
    context::AppContext,
    dao::{self, redis::OrderDao},
    solidity::{map_solidity_order_to_model, Orderbook::{OrderCreated, OrderFilled, OrderWithdrawn}},
};
use eyre::{Ok, Result};
use redis::Connection;
use slog::{info, warn};
use alloy::{primitives::Log, sol_types::SolEvent};
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
};
use futures_util::StreamExt;

use crate::solidity::Orderbook;


pub async fn process_order_withdrawn_log(
    context: &AppContext,
    log: Log<OrderWithdrawn>,
) -> Result<()> {
    let mut user_impl = dao::redis::UserImpl::new(context);
    user_impl.delete_order(log.orderId.to_vec()).await?;
    return Ok(());
}

pub async fn process_order_filled_log(
    context: &AppContext,
    log: Log<OrderFilled>,
) -> Result<()> {
    // TODO Check for order existence and set it as filled
    let mut user_impl = dao::redis::UserImpl::new(context);
    user_impl.delete_order(log.orderId.to_vec()).await?;
    Ok(())
}

pub async fn process_order_created_log(
    context: &AppContext,
    log: Log<OrderCreated>,
) -> Result<()> {
    // TODO Check for order existence and skip if it already exists
    info!(context.logger, "Processing log..."; "log" => format!("{:?}", log));
    let mut user_impl = dao::redis::UserImpl::new(context);

    info!(context.logger, "map solidity to model..."; "log" => format!("{:?}", log));
    let new_order = map_solidity_order_to_model(log.orderId.to_vec(), &log.order)?;

    info!(context.logger, "creating order..."; "log" => format!("{:?}", log));
    let _result = user_impl.create_order(&new_order).await?;
    info!(context.logger, "Processed log!"; "log" => format!("{:?}", log));
    
    Ok(())
}



pub async fn process_event_log(context: &AppContext, log: alloy::rpc::types::Log) -> Result<()> {
    info!(context.logger, "Processing Event Log"; "log" => format!("{:?}", log.data()));
    let order_created = Orderbook::OrderCreated::decode_log(&log.inner, false);
    if order_created.is_ok() {
        let order_created = order_created.unwrap();
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_created));
        process_order_created_log(context, order_created.clone()).await?;
        info!(context.logger, "Successfully processed log"; "log" => format!("{:?}", order_created));
        return Ok(());
    }

    let order_filled = Orderbook::OrderFilled::decode_log(&log.inner, false);
    if order_filled.is_ok() {
        let order_filled = order_filled.unwrap();        
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_filled));
        process_order_filled_log(context, order_filled.clone()).await?;
        info!(context.logger, "Successfully processed log"; "log" => format!("{:?}", order_filled));
        return Ok(());
    }

    let order_withdrawn = Orderbook::OrderWithdrawn::decode_log(&log.inner, false);
    if order_withdrawn.is_ok() {
        let order_withdrawn = order_withdrawn.unwrap();
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_withdrawn));
        process_order_withdrawn_log(context, order_withdrawn.clone()).await?;
        info!(context.logger, "Successfully processed log"; "log" => format!("{:?}", order_withdrawn));
        return Ok(());
    }

    warn!(context.logger, "Unable to decode log"; "log" => format!("{:?}", log));
    Err(eyre::eyre!("Unable to decode log"))
}

pub async fn run_event_listener_subscription_worker(context: &AppContext) -> Result<()> {
    // TODO FIXME: Refactor to subscribe to blocks and save processed blocks into DB
    //   or use a more efficient way to process logs, 
    //   but we need to keep track of the last processed ones and avoid duplicates/out of order processing
    let url = &context.config.ws_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);

    info!(context.logger, "Subscription worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let filter = Filter::new()
        .address(address)
        .events([
            Orderbook::OrderCreated::SIGNATURE,
            Orderbook::OrderWithdrawn::SIGNATURE,
            Orderbook::OrderFilled::SIGNATURE,
        ])
        .from_block(from_block);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    info!(context.logger, "Reading logs...");
    while let Some(log) = stream.next().await {
        process_event_log(context, log).await?;
    }

    info!(context.logger, "Subscription routine terminated!");
    Ok(())
}

pub async fn run_event_listener_poll_worker(context: &AppContext) -> Result<()> {
    // TODO FIXME: Save processed blocks into DB
    let url = &context.config.rpc_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);
    info!(context.logger, "Poll worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let filter = Filter::new()
        .address(address)
        .events([
            Orderbook::OrderCreated::SIGNATURE,
            Orderbook::OrderWithdrawn::SIGNATURE,
            Orderbook::OrderFilled::SIGNATURE,
        ])
        .from_block(from_block);

    info!(context.logger, "Reading logs...");
    let sub = provider.get_logs(&filter).await?;

    for log in sub {
        process_event_log(context, log).await?;
    }

    Ok(())
}
