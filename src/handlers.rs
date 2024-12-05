use tokio::sync::RwLock;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::SqlitePool;

use subxt::{OnlineClient, PolkadotConfig};
use subxt::blocks::Block;
use subxt::utils::AccountId32;

use crate::database::db_tools::insert_hash;
use crate::runtime;

pub async fn handle_block(client: Arc<OnlineClient<PolkadotConfig>>,
                          blocks_pool: Arc<SqlitePool>,
                          balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>,
                          block: Block<PolkadotConfig, OnlineClient<PolkadotConfig>>) {

    match insert_hash(blocks_pool, &block.header().number, block.hash().to_string().as_str()).await {
        Ok(res) => {
            if res { println!("#{} hash:{} was added", block.header().number, block.hash()) }
            else { println!("{} wasn't added", block.hash()) }
        },
        Err(e) => eprintln!("Error: {e}")
    }

    let rw_guard = balances.read().await;
    let tracking_addresses: Vec<String> = rw_guard.keys().cloned().collect();
    drop(rw_guard);

    for tracking_address in tracking_addresses {
        let account_id: AccountId32;

        match AccountId32::from_str(tracking_address.as_str()) {
            Ok(id) => account_id = id,
            Err(_) => {
                eprintln!("Incorrect account address: {}", tracking_address);
                continue;
            }
        }

        let storage_query = runtime::storage().system().account(&account_id);
        match client.storage().at(block.hash()).fetch(&storage_query).await {
            Ok(maybe_account) => {
                if let Some(account) = maybe_account {
                    let mut rw_guard = balances.write().await;
                    let block_num_to_balance = rw_guard.get_mut(&tracking_address).unwrap();
                    block_num_to_balance.insert(block.header().number, account.data.free);
                }
                else {
                    println!("No data for {}", account_id);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Fetching data from storage error: {}", e);
                continue;
            }
        }
    }
}