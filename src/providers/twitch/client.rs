use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use twitch_api::{
    HelixClient,
    eventsub::{EventSubscription, Transport},
};

use super::auth::UserTokenManager;
use crate::error::{AppError, AppResult};

pub struct TwitchApiClient {
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    token_manager: Arc<UserTokenManager>,
    needs_reauth: AtomicBool,
}

impl TwitchApiClient {
    pub fn new(
        helix: Arc<HelixClient<'static, reqwest::Client>>,
        token_manager: Arc<UserTokenManager>,
    ) -> Self {
        Self {
            helix,
            token_manager,
            needs_reauth: AtomicBool::new(false),
        }
    }

    pub async fn get_token(&self) -> Option<Arc<twitch_api::twitch_oauth2::UserToken>> {
        self.token_manager.get_token().await
    }

    pub async fn get_broadcaster_id(&self) -> Option<String> {
        self.token_manager.get_broadcaster_id().await
    }

    pub fn needs_reauth(&self) -> bool {
        self.needs_reauth.load(Ordering::Relaxed)
    }

    pub fn set_needs_reauth(&self, value: bool) {
        self.needs_reauth.store(value, Ordering::Relaxed);
    }

    pub async fn get_oauth_url(&self) -> AppResult<String> {
        self.token_manager.get_oauth_url().await
    }

    pub async fn exchange_code<T: Db + ?Sized>(&self, db: &T, code: &str) -> AppResult<bool> {
        let scopes_valid = self.token_manager.exchange_code(db, code).await?;
        if !scopes_valid {
            self.set_needs_reauth(true);
        }
        Ok(scopes_valid)
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
            .token_manager
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
