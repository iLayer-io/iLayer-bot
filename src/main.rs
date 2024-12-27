use slog::{error, info, Logger};
use tokio::{self};
use eyre::Result;
use dotenv::dotenv;
use std::sync::Arc;

mod context;
mod worker;
mod solidity;
mod orm;
mod dao;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let app_context = Arc::new(context::context()?);
    info!(app_context.logger,"Main function is starting...");

    let worker_handle = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move {
            worker::run_ordercreated_subscription_worker(&config).await
        })
    };

    let poll_worker_handle = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move {
            worker::run_ordercreated_poll_worker(&config).await
        })
    }; 

    let (worker_result, poll_worker_result) = tokio::join!(worker_handle, poll_worker_handle);

    handle_result(&app_context.logger,"Subscription worker", worker_result);
    handle_result(&app_context.logger,"Poll worker", poll_worker_result);

    info!(app_context.logger, "Main function is exiting...");
    Ok(())
}

fn handle_result(log: &Logger ,worker_name: &str, result: std::result::Result<eyre::Result<()>, tokio::task::JoinError>) {
    match result.unwrap() {
        Ok(_) => info!(log,"{} finished successfully", worker_name),
        Err(err) => error!(log,"{} failed: {}", worker_name, err),
    }
}
