use crate::{error::AppResult, providers::token_repository::TokenRecord, token_manager::TokenEnum};

#[async_trait::async_trait]
pub trait TokenProvider: Send + Sync {
    async fn validate_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum>;
    async fn force_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum>;
    async fn exchange_token(&self, code: &str) -> AppResult<TokenEnum>;
}
