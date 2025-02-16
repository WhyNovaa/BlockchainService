use crate::exit;
use crate::models::balances::Balances;
use crate::models::client::URL;
use crate::models::database::addresses_pool::AddressesPool;
use crate::tools::db_tools::insert_address_if_not_exist;
use axum::routing::get;
use axum::{
    extract::Path,
    response::{IntoResponse, Json as AxumJson},
    routing::post,
    Extension, Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use subxt::utils::AccountId32;
use tokio::net::TcpListener;
use tracing::{error, info, instrument, span, Level};

pub struct Server {
    addr_pool: AddressesPool,
    balances: Balances,
}

impl Server {
    pub fn new(addr_pool: AddressesPool, balances: Balances) -> Self {
        Server {
            addr_pool,
            balances,
        }
    }

    pub fn start(self, url: URL) {
        let span = span!(Level::INFO, "Server");

        tokio::spawn(async move {
            let _ = span.enter();
            info!("Starting router");
            let app = Router::new()
                .route("/api/balances/:address", post(add_address))
                .layer(Extension((self.addr_pool.pool(), self.balances.balances())))
                .route("/api/balances/:address/:block_no", get(get_balance))
                .layer(Extension(self.balances.balances()));

            info!("Starting listener");
            let listener = match TcpListener::bind(url.to_string()).await {
                Ok(listener) => listener,
                Err(e) => {
                    error!("Listener initialization error: {e}");
                    exit(1);
                }
            };

            info!("Serving listener and router");
            match axum::serve(listener, app).await {
                Ok(()) => {}
                Err(e) => {
                    error!("Service start error: {e}");
                    exit(1);
                }
            };
        });
    }
}

#[instrument]
async fn add_address(
    Extension((pool, balances)): Extension<(AddressesPool, Balances)>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    match AccountId32::from_str(address.as_str()) {
        Ok(_) => {}
        Err(_) => return AxumJson(json!({ "status": "400", "message": "Wrong address." })),
    }
    match insert_address_if_not_exist(pool, address.clone()).await {
        Ok(res) => {
            if res {
                let mut rw_guard = balances.0.write().await;
                rw_guard.insert(address, HashMap::new());
                AxumJson(json!({ "status": "201", "message": "Address added." }))
            } else {
                AxumJson(
                    json!({ "status": "400", "message": "The address is already being tracked." }),
                )
            }
        }
        Err(_) => AxumJson(json!({ "status": "500", "message": "Ooops, something went wrong." })),
    }
}

#[instrument]
async fn get_balance(
    Extension(balances): Extension<Balances>,
    Path((address, block_no)): Path<(String, u32)>,
) -> impl IntoResponse {
    let rw_guard = balances.0.read().await;
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
