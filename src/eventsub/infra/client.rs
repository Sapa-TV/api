use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use twitch_api::{HelixClient, eventsub::EventSubscription};

use crate::error::{AppError, AppResult};
use crate::token_manager::application::TokenManagerS;
use crate::token_manager::domain::types::{AccountVariant, ProviderVariant};

#[async_trait::async_trait]
pub trait TwitchApiClientTrait: Send + Sync {
    async fn get_oauth_url(&self) -> AppResult<String>;
    async fn exchange_code(&self, code: &str) -> AppResult<bool>;
}

pub struct TwitchApiClient {
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    token_manager: Arc<TokenManagerS>,
    needs_reauth: AtomicBool,
}

impl TwitchApiClient {
    pub fn new(
        helix: Arc<HelixClient<'static, reqwest::Client>>,
        token_manager: Arc<TokenManagerS>,
    ) -> Self {
        Self {
            helix,
            token_manager,
            needs_reauth: AtomicBool::new(false),
        }
    }

    pub async fn get_oauth_url(&self) -> AppResult<String> {
        self.token_manager
            .generate_url(ProviderVariant::Twitch)
            .await
    }

    pub async fn exchange_code(&self, code: &str) -> AppResult<bool> {
        let token_enum = self
            .token_manager
            .exchange_token(ProviderVariant::Twitch, AccountVariant::Main, code)
            .await?;
        if let crate::token_manager::domain::enums::TokenEnum::Twitch { .. } = token_enum {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_broadcaster_id(&self) -> Option<String> {
        None
    }

    pub async fn create_eventsub_subscription<E: EventSubscription + Send>(
        &self,
        subscription: E,
        transport: twitch_api::eventsub::Transport,
    ) -> Result<String, AppError> {
        let token = self
            .get_token()
            .await
            .ok_or_else(|| AppError::Internal("No token available".to_string()))?;

        let result = self
            .helix
            .create_eventsub_subscription(subscription, transport, &*token)
            .await
            .map_err(|e| {
                tracing::warn!("EventSub subscription failed: {}", e);
                self.set_needs_reauth(true);
                AppError::Internal(format!("Failed to create subscription: {}", e))
            })?;

        Ok(result.id.to_string())
    }

    async fn get_token(&self) -> Option<Arc<twitch_api::twitch_oauth2::UserToken>> {
        None
    }

    pub fn needs_reauth(&self) -> bool {
        self.needs_reauth.load(Ordering::Relaxed)
    }

    pub fn set_needs_reauth(&self, value: bool) {
        self.needs_reauth.store(value, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl TwitchApiClientTrait for TwitchApiClient {
    async fn get_oauth_url(&self) -> AppResult<String> {
        self.get_oauth_url().await
    }

    async fn exchange_code(&self, code: &str) -> AppResult<bool> {
        self.exchange_code(code).await
    }
}
