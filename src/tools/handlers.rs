use std::str::FromStr;

use subxt::{OnlineClient, PolkadotConfig};
use subxt::blocks::Block;
use subxt::utils::AccountId32;

use crate::modules::balances::Balances;
use crate::modules::client::Client;
use crate::modules::database::blocks_pool::BlocksPool;
use crate::tools::db_tools::insert_hash;
use crate::runtime;

pub async fn handle_block(client: Client,
                          blocks_pool: BlocksPool,
                          balances: Balances,
                          block: Block<PolkadotConfig, OnlineClient<PolkadotConfig>>) {

    handle_hash(
        blocks_pool.pool(),
        &block.header().number,
        block.hash().to_string().as_str()
    ).await;

    handle_accounts_in_hash(
        client,
        balances,
        block
    ).await;
}

pub async fn handle_hash(blocks_pool: BlocksPool, block_number: &u32, hash: &str) {
    match insert_hash(blocks_pool, block_number, hash).await {
        Ok(res) => {
            if res { println!("#{} hash:{} was added", block_number, hash) }
            else { println!("{} wasn't added", hash) }
        },
        Err(e) => eprintln!("Error: {e}")
    }
}

pub async fn handle_accounts_in_hash(client: Client,
                                     balances: Balances,
                                     block: Block<PolkadotConfig, OnlineClient<PolkadotConfig>>) {
    let rw_guard = balances.0.read().await;
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
                    let mut rw_guard = balances.0.write().await;
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