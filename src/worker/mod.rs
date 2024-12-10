use alloy::{primitives::address, providers::{Provider, ProviderBuilder}, rpc::types::Filter};
use eyre::Result;
use futures_util::StreamExt;

pub async fn run_subscription_worker()-> Result<()> {
    println!("Worker routine is running...");
    let rpc_url = "ws://127.0.0.1:8545";
    let address = address!("700b6A60ce7EaaEA56F065753d8dcB9653dbAD35");
    let from_block= 0;
    let event_name = "OrderCreated(uint256,uint256,uint256)";

    let provider = ProviderBuilder::new()
    .on_builtin(rpc_url)
    .await?;

    let filter = Filter::new()
        .address(address)
        .event(event_name)
        .from_block(from_block);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    println!("Reading logs...");
    while let Some(log) = stream.next().await {
        println!("Processing: {log:?}");
    }

    println!("Worker routine terminated!");
    Ok(())

}

pub async fn run_poll_worker()-> Result<()> {
    println!("Worker routine is running...");
    let rpc_url = "http://127.0.0.1:8545";
    let address = address!("700b6A60ce7EaaEA56F065753d8dcB9653dbAD35");
    let from_block= 0;
    let event_name = "OrderCreated(uint256,uint256,uint256)";

    let provider = ProviderBuilder::new()
    .on_builtin(rpc_url)
    .await?;

    let filter = Filter::new()
        .address(address)
        .event(event_name)
        .from_block(from_block);

    println!("Reading logs...");
    let sub = provider.get_logs(&filter).await?;
    for log in sub {
        println!("Processing: {log:?}");
    }

    println!("Worker routine terminated!");
    Ok(())
}