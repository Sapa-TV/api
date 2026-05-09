use std::sync::Arc;

use crate::error::AppResult;
use crate::oauth::domain::{OAuthApiClient, OAuthService};

pub struct TwitchOAuthService {
    client: Arc<dyn OAuthApiClient>,
}

impl TwitchOAuthService {
    pub fn new(client: Arc<dyn OAuthApiClient>) -> Self {
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
