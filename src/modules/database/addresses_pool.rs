use std::sync::Arc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AddressesPool(pub(crate) Arc<SqlitePool>);

impl AddressesPool {
    pub fn pool(&self) -> Self {
        Self(
            Arc::clone(&self.0)
        )
    }
}
