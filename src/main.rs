use tokio;
use eyre::Result;

mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Main function is running...");
    tokio::spawn(async {
        worker::run_worker().await
    });

    // Keep the main thread alive to allow worker to run
    tokio::signal::ctrl_c().await.unwrap();
    println!("\nExiting...");
    Ok(())
}
