use eyre::Result;
use tracing::{debug, error};

pub trait Service {
    async fn run(&self) {
        loop {
            match self._run().await {
                Ok(()) => debug!(
                    service = self.service_name(),
                    "Service stopped unexpectedly...",
                ),
                Err(e) => error!(
                    error = %e,
                    service = self.service_name(),
                    "Service error!"
                ),
            }

            // TODO Maybe we should make this configurable?
            tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        }
    }

    async fn _run(&self) -> Result<()>;

    fn service_name(&self) -> String;
}
