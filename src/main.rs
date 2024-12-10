use tokio::{self};
use eyre::Result;
use log::{info, error};
use dotenv::dotenv;
use env_logger;
use std::sync::Arc;

mod config;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let config = Arc::new(config::config()?);
    info!("Main function is starting...");

    let worker_handle = {
        let config = Arc::clone(&config);
        tokio::spawn(async move {
            worker::run_subscription_worker(&config).await
        })
    };

    let poll_worker_handle = {
        let config = Arc::clone(&config);
        tokio::spawn(async move {
            worker::run_poll_worker(&config).await
        })
    };

    let (worker_result, poll_worker_result) = tokio::join!(worker_handle, poll_worker_handle);

    handle_result("Subscription worker", worker_result);
    handle_result("Poll worker", poll_worker_result);

    info!("Main function is exiting...");
    Ok(())
}

fn handle_result(worker_name: &str, result: std::result::Result<eyre::Result<()>, tokio::task::JoinError>) {
    match result.unwrap() {
        Ok(_) => info!("{} finished successfully", worker_name),
        Err(err) => error!("{} failed: {}", worker_name, err),
    }
}
