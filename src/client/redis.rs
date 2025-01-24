use eyre::Result;
use redis::AsyncCommands;

pub(crate) static CHANNEL: &str = "orders";

pub(crate) async fn publish(
    conn: &mut redis::aio::MultiplexedConnection,
    order: entity::order::Model,
    chain_id: u64,
) -> Result<()> {
    let channel = format!("{}:{}", CHANNEL, chain_id);
    let message = serde_json::to_string(&order)?;
    let _: () = conn
        .publish(channel, message)
        .await
        .map_err(eyre::Error::from)?;
    Ok(())
}
