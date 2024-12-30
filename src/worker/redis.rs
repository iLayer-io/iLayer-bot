use eyre::Result;
use slog::info;

use crate::context::AppContext;

pub async fn run_order_filler_worker(context: &AppContext) -> Result<()> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(context.config.redis_poll_interval)).await;
        info!(context.logger, "Polling for ready orders to fill...");
    }
}