use crate::error::AppResult;
use crate::token_manager_v2::domain::enums::TokenEnum;
use crate::token_manager_v2::domain::types::TokenRecord;

#[async_trait::async_trait]
pub trait TokenProvider: Send + Sync {
    async fn validate_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum>;
    async fn force_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum>;
    async fn exchange_token(&self, code: &str) -> AppResult<TokenEnum>;
    fn generate_url(&self) -> AppResult<String>;
}
