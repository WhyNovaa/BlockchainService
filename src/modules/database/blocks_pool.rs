use std::sync::Arc;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct BlocksPool(pub(crate) Arc<SqlitePool>);

impl BlocksPool {
    pub fn pool(&self) -> Self {
        Self(
            Arc::clone(&self.0)
        )
    }
}