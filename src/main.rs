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


    // TODO FIXME Refactor, tokio should be used better here, it returns Ok(Err()) if the worker fails
    let poll_worker_handle: tokio::task::JoinHandle<Result<(), eyre::Report>> = {
        let config: Arc<context::AppContext> = Arc::clone(&app_context);
        tokio::spawn(async move { 
            worker::solidity::run_event_listener_poll_worker(&config).await?;
            Ok(())
        })
    };

    let poll_result = tokio::select! {
        res = poll_worker_handle => res,
    };

    match poll_result {
        Ok(Ok(())) => {
            info!(app_context.logger, "Poll worker has terminated!");
        },
        Ok(Err(e)) => {
            error!(app_context.logger, "Poll worker encountered an error: {:?}", e);
            return Err(e.into());
        },
        Err(e) => {
            error!(app_context.logger, "Poll worker encountered an error: {:?}", e);
            return Err(e.into());
        }
    }

    let event_subscription_worker_handle: tokio::task::JoinHandle<Result<(), eyre::Report>> = {
        let config = Arc::clone(&app_context);
        tokio::spawn(async move {
            worker::solidity::run_event_listener_subscription_worker(&config).await?;
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
