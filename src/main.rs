use dotenv::dotenv;
use eyre::Result;
use filler::Filler;
use service::Service;
use tokio::{self, task::JoinSet};
use tracing::info;
use tracing_subscriber::EnvFilter;
use watcher::Watcher;

mod context;
mod filler;
mod listener;
mod repository;
mod service;
mod solidity;
mod watcher;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let app_context = context::context()?;
    info!("Bot is starting");

    let mut join_set = JoinSet::new();

    for chain in &app_context.config.chain {
        info!(chain_name = chain.name, "Starting services");
        let listener = listener::Listener::new(
            app_context.config.postgres_url.clone(),
            app_context.config.redis_url.clone(),
            chain.clone(),
        );
        join_set.spawn(async move { listener.await.unwrap().run().await });

        let filler = Filler::new(
            app_context.config.postgres_url.clone(),
            app_context.config.redis_url.clone(),
            chain.clone(),
        );
        join_set.spawn(async move { filler.await.unwrap().run().await });

        let watcher = Watcher::new(
            app_context.config.postgres_url.clone(),
            app_context.config.redis_url.clone(),
            chain.clone(),
        );
        join_set.spawn(async move { watcher.await.unwrap().run().await });
    }

    while let Some(res) = join_set.join_next().await {
        res?;
    }

    Ok(())
}
