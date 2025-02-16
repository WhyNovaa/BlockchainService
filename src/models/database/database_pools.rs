use crate::models::database::addresses_pool::AddressesPool;
use crate::models::database::blocks_pool::BlocksPool;
use crate::tools::db_tools::{
    create_paths_if_necessary, ADDRESSES_NAME, ADDRESSES_STRUCT, BLOCKS_HISTORY_NAME,
    BLOCKS_HISTORY_STRUCT,
};
use sqlx::{Executor, SqlitePool};
use std::sync::Arc;
use tracing::info;

pub struct DatabasePools {
    blocks_pool: BlocksPool,
    addresses_pool: AddressesPool,
}

impl DatabasePools {
    pub async fn initialize() -> Result<Self, sqlx::Error> {
        create_paths_if_necessary().await;

        info!("Connecting to databases");
        let blocks_hist_conn = format!("sqlite:./{}.db", BLOCKS_HISTORY_NAME);
        let blocks_hist_pool = SqlitePool::connect(blocks_hist_conn.as_str()).await?;

        let addresses_conn = format!("sqlite:./{}.db", ADDRESSES_NAME);
        let addresses_pool = SqlitePool::connect(addresses_conn.as_str()).await?;

        blocks_hist_pool.execute(BLOCKS_HISTORY_STRUCT).await?;
        addresses_pool.execute(ADDRESSES_STRUCT).await?;

        Ok(Self {
            blocks_pool: BlocksPool(Arc::new(blocks_hist_pool)),
            addresses_pool: AddressesPool(Arc::new(addresses_pool)),
        })
    }

    pub fn blocks_pool(&self) -> BlocksPool {
        self.blocks_pool.pool()
    }

    pub fn addresses_pool(&self) -> AddressesPool {
        self.addresses_pool.pool()
    }
}
