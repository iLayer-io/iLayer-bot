use tokio::{self, join};
use eyre::Result;
use log::{info, error};
use dotenv::dotenv;
use env_logger;

mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    info!("Main function is starting...");

    let worker_handle = tokio::spawn(worker::run_subscription_worker());
    let poll_worker_handle = tokio::spawn(worker::run_poll_worker());

    let (worker_result, poll_worker_result) = join!(worker_handle, poll_worker_handle);

    handle_result("Subscription worker", worker_result);
    handle_result("Poll worker", poll_worker_result);

    info!("Main function is exiting...");
    Ok(())
}

fn handle_result(worker_name: &str, result: std::result::Result<eyre::Result<()>, tokio::task::JoinError>) {
    match result.unwrap() {
        Ok(_) => info!("{} finished successfully", worker_name),
        Err(err) => error!("{} failed: {}", worker_name, err),
    }
}
