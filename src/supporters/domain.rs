use crate::error::AppResult;

pub struct SupporterRepositoryData {
    pub king: String,
    pub day_supporters: Vec<String>,
    pub month_supporters: Vec<String>,
}

#[async_trait::async_trait]
pub trait SupporterRepository: Send + Sync {
    async fn init_supporters(&self, data: SupporterRepositoryData) -> AppResult<()>;

    async fn get_king(&self) -> AppResult<Option<String>>;
    async fn insert_king(&self, name: &str) -> AppResult<()>;

    async fn get_day_supporters(&self) -> AppResult<Vec<String>>;
    async fn insert_day_supporter(&self, name: &str) -> AppResult<()>;

    async fn get_month_supporters(&self) -> AppResult<Vec<String>>;
    async fn insert_month_supporter(&self, name: &str) -> AppResult<()>;
}
