use crate::error::AppResult;

#[async_trait::async_trait]
pub trait StateRepository: Send + Sync {
    async fn get_king(&self) -> AppResult<Option<String>>;
    async fn set_king(&self, name: &str) -> AppResult<()>;
    async fn get_day_supporters(&self) -> AppResult<Vec<String>>;
    async fn add_day_supporter(&self, name: &str) -> AppResult<()>;
    async fn get_month_supporters(&self) -> AppResult<Vec<String>>;
    async fn add_month_supporter(&self, name: &str) -> AppResult<()>;
}
