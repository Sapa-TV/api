use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use twitch_api::{
    HelixClient,
    eventsub::{EventSubscription, Transport},
    twitch_oauth2::TwitchToken,
};

use crate::error::{AppError, AppResult};
use crate::token_manager_v2::domain::types::{AccountVariant, ProviderVariant};

#[async_trait::async_trait]
pub trait TwitchApiClientTrait: Send + Sync {
    async fn get_token(&self) -> Option<Arc<twitch_api::twitch_oauth2::UserToken>>;
    async fn get_broadcaster_id(&self) -> Option<String>;
    fn needs_reauth(&self) -> bool;
    fn set_needs_reauth(&self, value: bool);
    async fn get_oauth_url(&self) -> AppResult<String>;
    async fn exchange_code(&self, code: &str) -> AppResult<bool>;
    fn subscribe_token_changes(&self) -> broadcast::Receiver<()>;
    async fn create_eventsub_subscription<E: EventSubscription + Send>(
        &self,
        subscription: E,
        transport: Transport,
    ) -> Result<String, AppError>;
}

pub struct TwitchApiClient {
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    token_manager: Arc<dyn TokenManagerTrait>,
    needs_reauth: AtomicBool,
}

#[async_trait::async_trait]
pub trait TokenManagerTrait: Send + Sync {
    async fn generate_url(&self, provider: ProviderVariant) -> AppResult<String>;
    async fn exchange_token(
        &self,
        provider: ProviderVariant,
        account: AccountVariant,
        code: &str,
    ) -> AppResult<crate::token_manager_v2::domain::enums::TokenEnum>;
    fn subscribe_token_changes(&self) -> broadcast::Receiver<()>;
}

impl TwitchApiClient {
    pub fn new(
        helix: Arc<HelixClient<'static, reqwest::Client>>,
        token_manager: Arc<dyn TokenManagerTrait>,
    ) -> Self {
        Self {
            helix,
            token_manager,
            needs_reauth: AtomicBool::new(false),
        }
    }

    pub async fn get_broadcaster_id(&self) -> Option<String> {
        None
    }

    pub fn needs_reauth(&self) -> bool {
        self.needs_reauth.load(Ordering::Relaxed)
    }

    pub fn set_needs_reauth(&self, value: bool) {
        self.needs_reauth.store(value, Ordering::Relaxed);
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
        if let crate::token_manager_v2::domain::enums::TokenEnum::Twitch { .. } = token_enum {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn subscribe_token_changes(&self) -> broadcast::Receiver<()> {
        self.token_manager.subscribe_token_changes()
    }

    pub async fn create_eventsub_subscription<E: EventSubscription + Send>(
        &self,
        subscription: E,
        transport: Transport,
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
}

#[async_trait::async_trait]
impl TwitchApiClientTrait for TwitchApiClient {
    async fn get_token(&self) -> Option<Arc<twitch_api::twitch_oauth2::UserToken>> {
        None
    }

    async fn get_broadcaster_id(&self) -> Option<String> {
        self.get_broadcaster_id().await
    }

    fn needs_reauth(&self) -> bool {
        self.needs_reauth()
    }

    fn set_needs_reauth(&self, value: bool) {
        self.set_needs_reauth(value);
    }

    async fn get_oauth_url(&self) -> AppResult<String> {
        self.get_oauth_url().await
    }

    async fn exchange_code(&self, code: &str) -> AppResult<bool> {
        self.exchange_code(code).await
    }

    fn subscribe_token_changes(&self) -> broadcast::Receiver<()> {
        self.subscribe_token_changes()
    }

    async fn create_eventsub_subscription<E: EventSubscription + Send>(
        &self,
        subscription: E,
        transport: Transport,
    ) -> Result<String, AppError> {
        self.create_eventsub_subscription(subscription, transport).await
    }
}
