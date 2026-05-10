use crate::error::AppResult;

#[async_trait::async_trait]
pub trait SupporterRepository: Send + Sync {
    async fn get_king(&self) -> AppResult<Option<String>>;
    async fn set_king(&self, name: &str) -> AppResult<()>;

    async fn get_day_supporters(&self) -> AppResult<Vec<String>>;
    async fn insert_day_supporter(&self, name: &str) -> AppResult<()>;

    async fn get_month_supporters(&self) -> AppResult<Vec<String>>;
    async fn insert_month_supporter(&self, name: &str) -> AppResult<()>;
}
