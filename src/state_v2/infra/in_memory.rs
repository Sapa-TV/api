use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppResult;
use crate::state_v2::domain::StateRepository;

#[derive(Clone)]
pub struct InMemoryStateRepository {
    king: Arc<RwLock<String>>,
    day_supporters: Arc<RwLock<Vec<String>>>,
    month_supporters: Arc<RwLock<Vec<String>>>,
}

impl InMemoryStateRepository {
    pub fn new() -> Self {
        Self {
            king: Arc::new(RwLock::new(String::new())),
            day_supporters: Arc::new(RwLock::new(Vec::new())),
            month_supporters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_data(king: String, day_supporters: Vec<String>, month_supporters: Vec<String>) -> Self {
        Self {
            king: Arc::new(RwLock::new(king)),
            day_supporters: Arc::new(RwLock::new(day_supporters)),
            month_supporters: Arc::new(RwLock::new(month_supporters)),
        }
    }
}

impl Default for InMemoryStateRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StateRepository for InMemoryStateRepository {
    async fn get_king(&self) -> AppResult<Option<String>> {
        let king = self.king.read().await;
        Ok(Some(king.clone()))
    }

    async fn set_king(&self, name: &str) -> AppResult<()> {
        let mut king = self.king.write().await;
        *king = name.to_string();
        Ok(())
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        let supporters = self.day_supporters.read().await;
        Ok(supporters.clone())
    }

    async fn add_day_supporter(&self, name: &str) -> AppResult<()> {
        let mut supporters = self.day_supporters.write().await;
        supporters.push(name.to_string());
        if supporters.len() > 10 {
            supporters.remove(0);
        }
        Ok(())
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        let supporters = self.month_supporters.read().await;
        Ok(supporters.clone())
    }

    async fn add_month_supporter(&self, name: &str) -> AppResult<()> {
        let mut supporters = self.month_supporters.write().await;
        supporters.push(name.to_string());
        Ok(())
    }
}
