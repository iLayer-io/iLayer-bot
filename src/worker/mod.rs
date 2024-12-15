use alloy::sol_types::SolEvent;
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
};
use eyre::Result;
use futures_util::StreamExt;
use slog::info;

use crate::context::AppContext;
use crate::solidity::OrderCreated;

pub async fn run_ordercreated_subscription_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.ws_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);

    info!(context.logger, "Subscription worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let filter = Filter::new()
        .address(address)
        .event(OrderCreated::SIGNATURE)
        .from_block(from_block);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    info!(context.logger, "Reading logs...");
    while let Some(log) = stream.next().await {
        let order_created = OrderCreated::decode_log_data(log.data(), false);
        info!(context.logger, "Worker processing log"; "order" => format!("{:?}", order_created), "log" => format!("{:?}", log));
    }

    info!(context.logger, "Routine terminated!");
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
        .event(OrderCreated::SIGNATURE)
        .from_block(from_block);

    info!(context.logger, "Reading logs...");
    let sub = provider.get_logs(&filter).await?;
    for log in sub {
        let order_created = OrderCreated::decode_log_data(log.data(), false);
        info!(context.logger, "Worker processing log"; "order" => format!("{:?}", order_created), "log" => format!("{:?}", log));
    }

    info!(context.logger, "Routine terminated!");
    Ok(())
}
