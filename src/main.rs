use dotenv::dotenv;
use eyre::Result;
use slog::{error, info, warn};
use std::sync::Arc;
use tokio::{self};

mod context;
mod dao;
mod solidity;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let app_context = Arc::new(context::context()?);
    info!(app_context.logger, "Main function is starting...");

    let event_subscription_worker_handle: tokio::task::JoinHandle<Result<(), eyre::Report>> = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move {
            worker::solidity::run_block_listener_poll_worker(&config).await?;
            Ok(())
        })
    };

    let order_filler_worker_handle = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move { 
            worker::filler::run_order_filler_worker(&config).await?;
            Ok(()) 
        })
    };

    let result = tokio::select! {
        res = order_filler_worker_handle => ("order_filler_worker", res),
        res = event_subscription_worker_handle => ("event_subscription_worker", res),
    };

    match result {
        (worker_name, Ok(Ok(_))) => {
            warn!(app_context.logger, "{} has terminated unexpectedly", worker_name);
            Ok(())
        },
        (worker_name, Ok(Err(e))) => {
            error!(
                app_context.logger,
                "{} encountered an error: {:?}", worker_name, e
            );
            Err(e.into())
        },
        (worker_name, Err(e)) => {
            error!(
                app_context.logger,
                "{} encountered an error: {:?}", worker_name, e
            );
            Err(e.into())
        },
    }
}
