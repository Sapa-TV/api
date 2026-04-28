use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::Db;
use crate::error::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub king: Arc<RwLock<String>>,
    pub last_day: Arc<RwLock<Vec<String>>>,
    pub month: Arc<RwLock<Vec<String>>>,
}

impl AppState {
    pub fn with_data(king: String, last_day: Vec<String>, month: Vec<String>) -> Self {
        Self {
            king: Arc::new(RwLock::new(king)),
            last_day: Arc::new(RwLock::new(last_day)),
            month: Arc::new(RwLock::new(month)),
        }
    }
}

pub async fn create_state<T: Db>(db: &T) -> AppResult<AppState>
where
    T: Db + Send + Sync,
{
    let king = db.get_king().await?.unwrap_or_default();
    let last_day = db.get_last_day_donaters().await?;
    let month = db.get_month_donaters().await?;

    Ok(AppState::with_data(king, last_day, month))
}
