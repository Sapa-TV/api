use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppResult;
use crate::state::domain::StateRepository;
use crate::supporters::domain::SupporterRepository;

#[derive(Clone)]
pub struct AppState {
    pub king: Arc<RwLock<String>>,
    pub day_supporters: Arc<RwLock<Vec<String>>>,
    pub month_supporters: Arc<RwLock<Vec<String>>>,
}

impl AppState {
    pub fn with_data(
        king: String,
        day_supporters: Vec<String>,
        month_supporters: Vec<String>,
    ) -> Self {
        Self {
            king: Arc::new(RwLock::new(king)),
            day_supporters: Arc::new(RwLock::new(day_supporters)),
            month_supporters: Arc::new(RwLock::new(month_supporters)),
        }
    }
}

pub async fn create_state<T>(db: &T) -> AppResult<AppState>
where
    T: SupporterRepository + Send + Sync,
{
    let king = db.get_king().await?.unwrap_or_default();
    let day_supporters = db.get_day_supporters().await?;
    let month_supporters = db.get_month_supporters().await?;

    Ok(AppState::with_data(king, day_supporters, month_supporters))
}
