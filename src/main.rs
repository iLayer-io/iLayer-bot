use dotenv::dotenv;
use eyre::Result;
use slog::{error, info};
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

    let poll_worker_handle = {
        let config: Arc<context::AppContext> = Arc::clone(&app_context);
        tokio::spawn(async move { worker::solidity::run_event_listener_poll_worker(&config).await })
    };

    let poll_result = tokio::try_join!(poll_worker_handle);
    if let Err(e) = poll_result {
        error!(
            app_context.logger,
            "Event Log polling worker encountered an error: {:?}", e
        );
        return Err(e.into());
    }

    let event_subscription_worker_handle = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move {
            worker::solidity::run_event_listener_subscription_worker(&config).await
        })
    };

    let order_filler_worker_handle = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move { worker::redis::run_order_filler_worker(&config).await })
    };

    let result = tokio::try_join!(order_filler_worker_handle, event_subscription_worker_handle);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
