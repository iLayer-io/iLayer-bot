use tokio::{self, join, task::JoinHandle};
use eyre::Result;

mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Main function is running...");
    let worker_handle: JoinHandle<eyre::Result<()>> = tokio::spawn(async {
        worker::run_worker().await
    });

    let result: (std::result::Result<std::result::Result<(), eyre::Error>, tokio::task::JoinError>,) = join!(worker_handle);
    match result.0.unwrap() {
        Ok(_) => {
            println!("Worker finished successfully");
        }
        Err(err) => {
            eprintln!("Worker failed: {}", err);
        }
    }
    // Keep the main thread alive to allow worker to run
    println!("\nExiting...");
    Ok(())
}
