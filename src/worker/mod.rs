use std::sync::Arc;

use ethers::{abi::{Abi, Address}, contract::{Contract, EthEvent}, providers::{Http, StreamExt, Provider}, types::U256};
use tokio;
use eyre::Result;

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "OrderCreated", abi = "OrderCreated(uint256,uint256,uint256)")]
pub struct OrderCreated {
    pub chain_id: U256, 
    pub coin_id: U256, 
    pub amount: U256, 
}


pub async fn run_worker()-> Result<()> {
    println!("Worker routine is running...");
    let provider = Provider::<Http>::try_from("http://127.0.0.1:8545").unwrap();
    let provider = Arc::new(provider);
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    let contract_address: Address = "0x82Dc47734901ee7d4f4232f398752cB9Dd5dACcC".parse().unwrap();
    let abi_bytes  = include_bytes!("../../abi/Counter.json");
    let abi = Abi::load(&abi_bytes[..]).expect("Failed to parse ABI");

    let contract: ethers::contract::ContractInstance<Arc<Provider<Http>>, _> = Contract::new(contract_address, abi, provider.clone());
    let event = contract.event::<OrderCreated>();
    
    // TODO FIXME Filter only events incoming from the contract address
    let mut event_stream = event.stream().await.unwrap();

    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => {
                println!("Received OrderCreated event: {:?}", event);
            }
            Err(err) => {
                eprintln!("Error while receiving event: {:?}", err);
            }
        }
    }

    println!("Listening for OrderCreated events...");
    
    Ok(())

}