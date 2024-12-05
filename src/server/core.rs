use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use axum::{extract::Path, response::{IntoResponse, Json as AxumJson}, routing::post, Extension, Router};
use axum::routing::get;
use tokio::net::TcpListener;
use crate::database::db_tools::insert_address_if_not_exist;

pub struct Server {
    addr_pool: Arc<SqlitePool>,
    balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>,
}

impl Server {
    pub fn new(addr_pool: Arc<SqlitePool>, balances: Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>) -> Self {
        Server { addr_pool, balances }
    }

    pub fn start(&self, addr: &str) {
        let addr_pool_clone1 = Arc::clone(&self.addr_pool);
        let balances_clone1 = Arc::clone(&self.balances);
        let balances_clone2 = Arc::clone(&self.balances);
        let addr = addr.to_string();

        tokio::spawn(async move {
            let app = Router::new()
                .route("/api/balances/:address", post(add_address))
                .layer(Extension((addr_pool_clone1, balances_clone1)))
                .route("/api/balances/:address/:block_no", get(get_balance))
                .layer(Extension(balances_clone2));

            let listener = TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    }
}


async fn add_address(
    Extension((pool, balances)): Extension<(Arc<SqlitePool>, Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>)>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    println!("{}", address);
    match insert_address_if_not_exist(pool, address.clone()).await {
        Ok(res) => {
            if res {
                let mut rw_guard = balances.write().await;
                rw_guard.insert(address, HashMap::new());
                AxumJson(json!({ "status": "201", "message": "Address added." }))
            } else {
                AxumJson(json!({ "status": "400", "message": "The address is already being tracked." }))
            }
        }
        Err(_) => AxumJson(json!({ "status": "500", "message": "Ooops, something went wrong." })),
    }
}

async fn get_balance(
    Extension(balances): Extension<Arc<RwLock<HashMap<String, HashMap<u32, u64>>>>>,
    Path((address, block_no)): Path<(String, u32)>,
) -> impl IntoResponse {

    let rw_guard = balances.read().await;
    let maybe_block_num_to_balance = rw_guard.get(&address).cloned();
    drop(rw_guard);

    if let Some(block_num_to_balance) = maybe_block_num_to_balance {
        if let Some(balance) = block_num_to_balance.get(&block_no) {
            AxumJson(json!({ "status": "200", "address": address, "balance": balance }))
        } else {
            AxumJson(json!({ "status": "202", "message": "Data wasn't indexed yet." }))
        }
    } else {
        AxumJson(json!({ "status": "404", "message": "Address wasn't found." }))
    }
}

