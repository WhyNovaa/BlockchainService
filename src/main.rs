mod models;
mod server;
mod tools;

use models::database::database_pools::DatabasePools;
use std::process::exit;
use tracing::error;

use crate::models::balances::Balances;
use crate::models::client::{Client, URL};
use crate::server::core::Server;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod runtime {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let pools = match DatabasePools::initialize().await {
        Ok(res) => res,
        Err(e) => {
            error!("Database initialization error: {e}");
            exit(1);
        }
    };

    let balances = match Balances::initialize(&pools).await {
        Ok(balances) => balances,
        Err(e) => {
            error!("Balances initialization error: {e}");
            exit(1);
        }
    };

    let client = match Client::initialize(URL("ws://127.0.0.1:9944")).await {
        Ok(cl) => cl,
        Err(e) => {
            error!("Client creation error: {e}");
            exit(1);
        }
    };

    let _ = Server::new(pools.addresses_pool(), balances.balances()).start(URL("localhost:8080"));

    client
        .start_subscription(pools.blocks_pool(), balances.balances())
        .await?;

    Ok(())
}
