use dotenv::dotenv;
use eyre::Result;
use futures_util::future;
use tokio::{self, task::JoinSet};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod context;
mod repository;
mod solidity;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    dotenv().ok();

    let app_context = context::context()?;
    info!("Bot is starting...");

    let mut set = JoinSet::new();

    for chain in &app_context.config.chain {
        info!("Starting worker for chain: {}", chain.name);
        let worker = worker::Worker::new(app_context.config.postgres_url.clone(), chain.clone());
        set.spawn(async move {
            worker
                .await
                .unwrap()
                .run_block_listener_subscription()
                .await
        });
    }

    // Process futures dynamically
    while let Some(res) = set.join_next().await {
        res??;
    }

    Ok(())

    // let order_filler_worker_handle = worker::filler::run_order_filler_worker(&app_context);

    // let result = tokio::select! {
    //     // res = order_filler_worker_handle => ("order_filler_worker", res),
    //     res = block_subscription_handle => ("event_subscription_worker", res),
    // };

    // match result {
    //     (worker_name, Ok(_)) => {
    //         warn!("{worker_name} has terminated unexpectedly!");
    //         Ok(())
    //     }
    //     (worker_name, Err(e)) => {
    //         error!("{worker_name} encountered an error: {e:?}");
    //         Err(e.into())
    //     }
    // }
}
