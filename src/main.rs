mod database;
mod server;
mod handlers;

use tokio::sync::RwLock;

use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;

use sqlx::SqlitePool;

use subxt::{OnlineClient, PolkadotConfig};

use crate::database::db_tools::{connect_to_databases, load_addresses};
use crate::handlers::handle_block;
use crate::server::core::Server;


#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod runtime {}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let blocks_pool: Arc<SqlitePool>;
    let addresses_pool: Arc<SqlitePool>;

    match connect_to_databases().await {
        Ok(c) => {
            blocks_pool = Arc::new(c.0);
            addresses_pool = Arc::new(c.1);
        }
        Err(e) => {
            println!("Db connection error{:?}", e);
            exit(1);
        }
    }

    // HashMap<AccountId32.to_string(), HashMap<block_no, balance>>
    let balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>> = Arc::new(RwLock::new(HashMap::new()));

    match load_addresses(Arc::clone(&addresses_pool), Arc::clone(&balances)).await {
        Ok(()) =>  {
            println!("Balances loaded successfully");
            let loaded_bal = balances.read().await;
            println!("Loaded: {:?}", loaded_bal);
            drop(loaded_bal);
        },
        Err(e) => eprintln!("{e}")
    }


    let _ = Server::new(Arc::clone(&addresses_pool), Arc::clone(&balances))
        .start("localhost:8080");



    let client = Arc::new(OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?);

    let mut blocks = client.blocks().subscribe_finalized().await?;

    while let block = blocks.next().await.unwrap().unwrap() {

        let client = Arc::clone(&client);
        let blocks_pool = Arc::clone(&blocks_pool);
        let balances = Arc::clone(&balances);

        tokio::spawn(async move {
            handle_block(client, blocks_pool, balances, block).await
        }).await?;
    }

    Ok(())
}
