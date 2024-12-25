use alloy::rpc::types::Log;
use alloy::sol_types::{SolError, SolEvent};
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
};
use eyre::{Ok, Result};
use futures_util::StreamExt;
use slog::{debug, info, warn};

use crate::context::AppContext;
use crate::solidity::{OrderCreated, Orderbook};

mod process;


pub async fn process_log(context: &AppContext, log: Log) -> Result<()> {
    let order_created = Orderbook::OrderCreated::decode_log(&log.inner, false);
    if order_created.is_ok() {
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_created.unwrap()));
        process::process_order_created_log(context, log).await?;
        return Ok(());
    }

    let order_filled = Orderbook::OrderFilled::decode_log(&log.inner, false);
    if order_filled.is_ok() {
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_filled.unwrap()));
        return Ok(());
    }

    let order_withdrawn = Orderbook::OrderWithdrawn::decode_log(&log.inner, false);
    if order_withdrawn.is_ok() {
        info!(context.logger, "Successfully decoded log"; "log" => format!("{:?}", order_withdrawn.unwrap()));
        return Ok(());
    }

    warn!(context.logger, "Unable to decode log"; "log" => format!("{:?}", log));
    Err(eyre::eyre!("Unable to decode log"))
}

pub async fn run_ordercreated_subscription_worker(context: &AppContext) -> Result<()> {
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
        process_log(context, log).await?;
    }

    info!(context.logger, "Subscription routine terminated!");
    Ok(())
}

pub async fn run_ordercreated_poll_worker(context: &AppContext) -> Result<()> {
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
        process_log(context, log).await?;
    }

    Ok(())
}
