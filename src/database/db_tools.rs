use std::collections::HashMap;
use std::sync::Arc;
use sqlx::{sqlite::SqlitePool, Executor, Row};
use tokio::fs::{metadata, File};
use tokio::sync::RwLock;

const BLOCKS_HISTORY_NAME: &str = "BlocksHist";
const ADDRESSES_NAME: &str = "Addresses";

const BLOCKS_HISTORY_STRUCT: &str = "CREATE TABLE IF NOT EXISTS BlocksHist (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            number INT NOT NULL UNIQUE,
            hash TEXT NOT NULL
        )";

const ADDRESSES_STRUCT: &str = "CREATE TABLE IF NOT EXISTS Addresses (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            address TEXT NOT NULL UNIQUE
        )";

pub async fn connect_to_databases() -> Result<(SqlitePool, SqlitePool), sqlx::Error> {
    create_paths_if_necessary().await;

    let blocks_hist_conn = format!("sqlite:./{}.db", BLOCKS_HISTORY_NAME);
    let blocks_hist_pool = SqlitePool::connect(blocks_hist_conn.as_str()).await
        .expect("DB(BlocksHistory) pool initialization error");

    let addresses_conn = format!("sqlite:./{}.db", ADDRESSES_NAME);
    let addresses_pool = SqlitePool::connect(addresses_conn.as_str()).await
        .expect("DB(Addresses) pool initialization error");

    blocks_hist_pool.execute(BLOCKS_HISTORY_STRUCT)
        .await?;
    addresses_pool.execute(ADDRESSES_STRUCT)
        .await?;

    Ok((blocks_hist_pool, addresses_pool))
}

async fn create_paths_if_necessary() {
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

pub async fn insert_hash(pool: Arc<SqlitePool>, number: &u32, hash: &str) -> Result<bool, sqlx::Error> {
    let req = format!("INSERT INTO {}(number, hash) VALUES(?, ?)", BLOCKS_HISTORY_NAME);
    let res = sqlx::query(req.as_str())
        .bind(number)
        .bind(hash)
        .execute(&*pool)
        .await?;

    Ok(res.rows_affected() == 1)
}

pub async fn is_address_exists(pool: Arc<SqlitePool>, address: String) -> Result<bool, sqlx::Error> {
    let req = format!("SELECT 1 FROM {} WHERE address=?", ADDRESSES_NAME);

    let row = sqlx::query(req.as_str())
        .bind(address)
        .fetch_optional(&*pool)
        .await?;

    Ok(row.is_some())
}

pub async fn insert_address_if_not_exist(pool: Arc<SqlitePool>, address: String) -> Result<bool, sqlx::Error> {
    if is_address_exists(Arc::clone(&pool), address.clone()).await? {
        Ok(false)
    }
    else {
        let req = format!("INSERT INTO {}(address) VALUES(?)", ADDRESSES_NAME);

        let res = sqlx::query(req.as_str())
            .bind(address)
            .execute(&*pool)
            .await?;
        Ok(res.rows_affected() == 1)
    }
}

pub async fn load_addresses(pool: Arc<SqlitePool>, balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>) -> Result<(), sqlx::Error> {
    let req = format!("SELECT * FROM {}", ADDRESSES_NAME);

    let rows = sqlx::query(req.as_str())
        .fetch_all(&*pool)
        .await?;

    let mut rw_guard = balances.write().await;
    for row in rows {
        let addr = row.get("address");
        rw_guard.insert(addr, HashMap::new());
    }

    Ok(())
}
