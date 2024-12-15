use alloy::rpc::types::Log;
use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::Filter,
};
use eyre::Result;
use futures_util::StreamExt;
use slog::info;

use crate::context::AppContext;

const EVENT_SIG: &str = "OrderCreated(uint256,uint256,uint256)";

fn parse_log(log: &Log) -> (U256, U256, U256) {
    let data = &log.data().data;
    let param1 = U256::from_be_bytes::<32>(data[0..32].try_into().unwrap());
    let param2 = U256::from_be_bytes::<32>(data[32..64].try_into().unwrap());
    let param3 = U256::from_be_bytes::<32>(data[64..96].try_into().unwrap());
    (param1, param2, param3)
}

pub async fn run_ordercreated_subscription_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.ws_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);

    info!(context.logger, "Subscription worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block, "event_sig" => EVENT_SIG);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let filter = Filter::new()
        .address(address)
        .event(EVENT_SIG)
        .from_block(from_block);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    info!(context.logger, "Reading logs...");
    while let Some(log) = stream.next().await {
        let (param1, param2, param3) = parse_log(&log);
        info!(context.logger, "Worker processing log"; "param1" => format!("{:?}", param1), "param2" => format!("{:?}", param2), "param3" => format!("{:?}", param3));
    }

    info!(context.logger, "Routine terminated!");
    Ok(())
}

pub async fn run_ordercreated_poll_worker(context: &AppContext) -> Result<()> {
    let url = &context.config.rpc_url;
    let address: Address = context.config.order_contract_address.parse()?;
    let from_block = context.config.from_block.unwrap_or(0);
    info!(context.logger, "Poll worker routine is starting!"; "url" => url, "address" => format!("{:?}", address), "from_block" => from_block, "event_sig" => EVENT_SIG);

    let provider = ProviderBuilder::new().on_builtin(url).await?;

    let filter = Filter::new()
        .address(address)
        .event(EVENT_SIG)
        .from_block(from_block);

    info!(context.logger, "Reading logs...");
    let sub = provider.get_logs(&filter).await?;
    for log in sub {
        let (param1, param2, param3) = parse_log(&log);
        info!(context.logger, "Worker processing log"; "param1" => format!("{:?}", param1), "param2" => format!("{:?}", param2), "param3" => format!("{:?}", param3));
    }

    info!(context.logger, "Routine terminated!");
    Ok(())
}
