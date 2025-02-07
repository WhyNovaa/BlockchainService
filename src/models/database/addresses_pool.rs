use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AddressesPool(pub(crate) Arc<SqlitePool>);

impl AddressesPool {
    pub fn pool(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
