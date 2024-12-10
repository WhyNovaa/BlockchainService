use std::collections::HashMap;
use std::sync::Arc;
use sqlx::Row;
use tokio::fs::{metadata, File};
use tokio::sync::RwLock;
use crate::modules::database::{addresses_pool::AddressesPool, blocks_pool::BlocksPool};
pub const BLOCKS_HISTORY_NAME: &str = "BlocksHist";
pub const ADDRESSES_NAME: &str = "Addresses";

pub const BLOCKS_HISTORY_STRUCT: &str = "CREATE TABLE IF NOT EXISTS BlocksHist (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            number INT NOT NULL UNIQUE,
            hash TEXT NOT NULL
        )";

pub const ADDRESSES_STRUCT: &str = "CREATE TABLE IF NOT EXISTS Addresses (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            address TEXT NOT NULL UNIQUE
        )";

pub async fn create_paths_if_necessary() {
    create_path_if_necessary(BLOCKS_HISTORY_NAME).await;
    create_path_if_necessary(ADDRESSES_NAME).await;
}

async fn create_path_if_necessary(name: &str) {
    let file_path = format!("{}.db", name);
    match metadata(file_path.clone()).await {
        Ok(_) => {}
        Err(_) => {
            File::create(file_path).await.unwrap();
        }
    }
}

pub async fn insert_hash(blocks_pool: BlocksPool, number: &u32, hash: &str) -> Result<bool, sqlx::Error> {
    let req = format!("INSERT INTO {}(number, hash) VALUES(?, ?)", BLOCKS_HISTORY_NAME);
    let res = sqlx::query(req.as_str())
        .bind(number)
        .bind(hash)
        .execute(&*blocks_pool.0)
        .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn is_address_exists(addr_pool: AddressesPool, address: String) -> Result<bool, sqlx::Error> {
    let req = format!("SELECT 1 FROM {} WHERE address=?", ADDRESSES_NAME);

    let row = sqlx::query(req.as_str())
        .bind(address)
        .fetch_optional(&*addr_pool.0)
        .await?;

    Ok(row.is_some())
}

pub async fn insert_address_if_not_exist(addr_pool: AddressesPool, address: String) -> Result<bool, sqlx::Error> {
    if is_address_exists(addr_pool.pool(), address.clone()).await? {
        Ok(false)
    }
    else {
        let req = format!("INSERT INTO {}(address) VALUES(?)", ADDRESSES_NAME);

        let res = sqlx::query(req.as_str())
            .bind(address)
            .execute(&*addr_pool.0)
            .await?;
        Ok(res.rows_affected() == 1)
    }
}

pub async fn load_addresses(addr_pool: AddressesPool, balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>) -> Result<(), sqlx::Error> {
    let req = format!("SELECT * FROM {}", ADDRESSES_NAME);

    let rows = sqlx::query(req.as_str())
        .fetch_all(&*addr_pool.0)
        .await?;

    let mut rw_guard = balances.write().await;
    for row in rows {
        let addr = row.get("address");
        rw_guard.insert(addr, HashMap::new());
    }

    Ok(())
}
