use alloy::{primitives::{address, U256}, providers::{Provider, ProviderBuilder}, rpc::types::Filter};
use eyre::Result;
use futures_util::StreamExt;
use log::info;
use alloy::rpc::types::Log;

use crate::config::AppConfig;

fn parse_log(log: &Log) -> (U256, U256, U256) {
    let data = &log.data().data;
    let param1= U256::from_be_bytes::<32>(data[0..32].try_into().unwrap());
    let param2 = U256::from_be_bytes::<32>(data[32..64].try_into().unwrap());
    let param3 = U256::from_be_bytes::<32>(data[64..96].try_into().unwrap());
    (param1, param2, param3)
}

pub async fn run_subscription_worker(config: &AppConfig) -> Result<()> {
    let url = &config.ws_url;
    info!("Subscription worker routine is starting with URL: {}", url);
    let address = address!("700b6A60ce7EaaEA56F065753d8dcB9653dbAD35");
    let from_block = 0;
    let event_name = "OrderCreated(uint256,uint256,uint256)";

    let provider = ProviderBuilder::new()
        .on_builtin(url)
        .await?;

    let filter = Filter::new()
        .address(address)
        .event(event_name)
        .from_block(from_block);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    info!("Subscription worker is reading logs...");
    while let Some(log) = stream.next().await {
        let (param1, param2, param3) = parse_log(&log);
        info!("Poll worker processing log: {log:?}, param1: {param1}, param2: {param2}, param3: {param3}");
    }

    info!("Subscription worker routine terminated!");
    Ok(())
}

pub async fn run_poll_worker(config: &AppConfig)  -> Result<()> {
    let url = &config.rpc_url;
    info!("Poll worker routine is starting with URL: {}", url);
    let address = address!("700b6A60ce7EaaEA56F065753d8dcB9653dbAD35");
    let from_block = 0;
    let event_name = "OrderCreated(uint256,uint256,uint256)";

    let provider = ProviderBuilder::new()
        .on_builtin(url)
        .await?;

    let filter = Filter::new()
        .address(address)
        .event(event_name)
        .from_block(from_block);

    info!("Poll worker is reading logs...");
    let sub = provider.get_logs(&filter).await?;
    for log in sub {
        let (param1, param2, param3) = parse_log(&log);
        info!("Poll worker processing log: {log:?}, param1: {param1}, param2: {param2}, param3: {param3}");
    }

    info!("Poll worker routine terminated!");
    Ok(())
}