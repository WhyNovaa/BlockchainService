use crate::models::balances::Balances;
use crate::models::database::blocks_pool::BlocksPool;
use crate::tools::handlers::handle_block;
use std::process::exit;
use std::sync::Arc;
use subxt::blocks::BlocksClient;
use subxt::storage::StorageClient;
use subxt::{OnlineClient, PolkadotConfig};

pub struct URL(pub &'static str);
impl URL {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct Client(Arc<OnlineClient<PolkadotConfig>>);

impl Client {
    pub async fn initialize(url: URL) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Arc::new(OnlineClient::<PolkadotConfig>::from_url(url.0).await?);

        Ok(Self(client))
    }

    pub fn storage(&self) -> StorageClient<PolkadotConfig, OnlineClient<PolkadotConfig>> {
        self.0.storage()
    }
    pub fn blocks(&self) -> BlocksClient<PolkadotConfig, OnlineClient<PolkadotConfig>> {
        self.0.blocks()
    }

    pub async fn start_subscription(
        &self,
        blocks_pool: BlocksPool,
        balances: Balances,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocks = match self.blocks().subscribe_finalized().await {
            Ok(block_stream) => block_stream,
            Err(e) => {
                eprintln!("Error in getting BlockStream: {e}");
                exit(1);
            }
        };

        while let Some(block_res) = blocks.next().await {
            match block_res {
                Ok(block) => {
                    let client = self.client();
                    let blocks_pool = blocks_pool.pool();
                    let balances = balances.balances();

                    tokio::spawn(async move {
                        handle_block(client, blocks_pool, balances, block).await
                    })
                    .await?;
                }
                Err(e) => {
                    eprintln!("Error processing block: {}", e);
                    continue;
                }
            }
        }

        Ok(())
    }

    pub fn client(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
