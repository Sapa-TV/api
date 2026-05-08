use crate::{
    error::AppResult,
    providers::token_repository::TokenRecord,
    token_manager::{TokenEnum, TokenProvider},
};

pub struct TwitchTokenProvider {}

#[async_trait::async_trait]
impl TokenProvider for TwitchTokenProvider {
    async fn refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum> {
        todo!()
    }
    async fn exchange_token(&self, code: &str) -> AppResult<TokenEnum> {
        todo!()
    }
}
