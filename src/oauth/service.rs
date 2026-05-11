use async_trait::async_trait;

use crate::error::AppResult;

#[async_trait]
pub trait OAuthService: Send + Sync {
    async fn get_auth_url(&self) -> AppResult<String>;
    async fn handle_callback(&self, code: &str) -> AppResult<bool>;
}
