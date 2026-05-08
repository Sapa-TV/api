pub struct OAuthCallbackResult {
    pub success: bool,
}

#[async_trait::async_trait]
pub trait OAuthService: Send + Sync {
    async fn get_auth_url(&self) -> crate::error::AppResult<String>;
    async fn handle_callback(&self, code: &str) -> crate::error::AppResult<bool>;
}
