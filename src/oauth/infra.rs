use std::sync::Arc;

use crate::error::AppResult;
use crate::eventsub::domain::TwitchApiClientTrait;
use crate::oauth::domain::OAuthService;

pub struct TwitchOAuthService {
    client: Arc<dyn TwitchApiClientTrait>,
}

impl TwitchOAuthService {
    pub fn new(client: Arc<dyn TwitchApiClientTrait>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl OAuthService for TwitchOAuthService {
    async fn get_auth_url(&self) -> AppResult<String> {
        self.client.get_oauth_url().await
    }

    async fn handle_callback(&self, code: &str) -> AppResult<bool> {
        self.client.exchange_code(code).await
    }
}
