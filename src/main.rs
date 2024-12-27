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
    // TODO FIXME: We should implement the following workflow:
    // 1. Poll from block_height all the blocks:
    //    - Get all the logs from the block, and process them saving orders to DB
    //    - Save the block as processed in the database
    // 2. Start this two workers:
    //    2a. Subscribe to new blocks:
    //      - Get all the logs from the block, and process them saving orders to DB
    //      - Save the block as processed in the database
    //    2b. Subscribe to new orders (scheduled jobs?):
    //      - Check for orders that are not expired, after primary filler deadline, and try to fill them


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
