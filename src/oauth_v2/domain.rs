use crate::error::AppResult;

pub struct OAuthCallbackResult {
    pub success: bool,
    pub message: String,
}

#[async_trait::async_trait]
pub trait OAuthService: Send + Sync {
    async fn get_auth_url(&self) -> AppResult<String>;
    async fn exchange_code(&self, code: &str) -> AppResult<OAuthCallbackResult>;
}
