use dotenv::dotenv;
use eyre::Result;
use tokio::{self};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod context;
mod repository;
mod solidity;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let app_context = context::context()?;

    info!("Main function is starting...");

    let block_subscription_worker_handle =
        worker::run_block_listener_subscription_worker(&app_context);

    let order_filler_worker_handle = worker::filler::run_order_filler_worker(&app_context);

    let result = tokio::select! {
        res = order_filler_worker_handle => ("order_filler_worker", res),
        res = block_subscription_worker_handle => ("event_subscription_worker", res),
    };

    match result {
        (worker_name, Ok(_)) => {
            warn!("{} has terminated unexpectedly", worker_name);
            Ok(())
        }
        (worker_name, Err(e)) => {
            error!("{} encountered an error: {:?}", worker_name, e);
            Err(e.into())
        }
    }
}
