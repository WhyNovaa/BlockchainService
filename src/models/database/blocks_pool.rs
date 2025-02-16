use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BlocksPool(pub(crate) Arc<SqlitePool>);

impl BlocksPool {
    pub fn pool(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
