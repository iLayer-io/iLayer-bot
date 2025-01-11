use dotenv::dotenv;
use eyre::Result;
use filler::Filler;
use tokio::{self, task::JoinSet};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod context;
mod filler;
mod listener;
mod repository;
mod solidity;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    dotenv().ok();

    let app_context = context::context()?;
    info!("Bot is starting");

    let mut join_set = JoinSet::new();

    for chain in &app_context.config.chain {
        info!(chain_name = chain.name, "Starting services");
        let listener =
            listener::Listener::new(app_context.config.postgres_url.clone(), chain.clone());
        join_set.spawn(async move { listener.await.unwrap().run_subscription().await });

        let filler = Filler::new(app_context.config.postgres_url.clone(), chain.clone());

        join_set.spawn(async move { filler.await.unwrap().run().await });
    }

    while let Some(res) = join_set.join_next().await {
        res??;
    }

    Ok(())
}
