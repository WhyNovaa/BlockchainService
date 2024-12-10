use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::modules::database::database_pools::DatabasePools;
use crate::tools::db_tools::load_addresses;

#[derive(Clone)]
pub struct Balances(pub(crate) Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>);

impl Balances {
    pub async fn initialize(pools: &DatabasePools) -> Result<Self, Box<dyn std::error::Error>>{
        let balances = Arc::new(RwLock::new(HashMap::new()));

        load_addresses(pools.addresses_pool(), Arc::clone(&balances)).await?;

        Ok(Self(
            balances
        ))
    }

    pub fn balances(&self) -> Balances {
        Self(
            Arc::clone(&self.0)
        )
    }
}