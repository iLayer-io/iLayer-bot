use tokio::{self, join, task::JoinHandle};
use eyre::Result;

mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Main function is running...");

    let worker_handle = tokio::spawn(worker::run_subscription_worker());
    let poll_worker_handle = tokio::spawn(worker::run_poll_worker());

    let (worker_result, poll_worker_result) = join!(worker_handle, poll_worker_handle);

    handle_result("Subscription worker", worker_result);
    handle_result("Poll worker", poll_worker_result);

    println!("\nExiting...");
    Ok(())
}

fn handle_result(worker_name: &str, result: std::result::Result<eyre::Result<()>, tokio::task::JoinError>) {
    match result.unwrap() {
        Ok(_) => println!("{} finished successfully", worker_name),
        Err(err) => eprintln!("{} failed: {}", worker_name, err),
    }
}
