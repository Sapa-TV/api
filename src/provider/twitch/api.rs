use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use twitch_api::twitch_oauth2::{AccessToken, ClientSecret, RefreshToken, UserToken};
use twitch_api::{HelixClient, eventsub::EventSubscription};

use crate::error::{AppError, AppResult};
use crate::oauth::service::OAuthService;
use crate::token::manager::TokenManager;
use crate::token::types::{AccountVariant, ProviderVariant};

pub struct TwitchApiClient {
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    token_manager: Arc<TokenManager>,
    client_secret: String,
    needs_reauth: AtomicBool,
}

impl TwitchApiClient {
    pub fn new(
        helix: Arc<HelixClient<'static, reqwest::Client>>,
        token_manager: Arc<TokenManager>,
        client_secret: String,
    ) -> Self {
        Self {
            helix,
            token_manager,
            client_secret,
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
        #[allow(irrefutable_let_patterns)]
        if let crate::token::enums::TokenEnum::Twitch { .. } = token_enum {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_broadcaster_id(&self) -> Option<String> {
        let token_record = self
            .token_manager
            .get_token(ProviderVariant::Twitch, AccountVariant::Main)
            .await
            .ok()?;

        match &token_record.token {
            crate::token::enums::TokenEnum::Twitch { user_id, .. } => Some(user_id.clone()),
        }
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

    async fn get_token(&self) -> Option<Arc<UserToken>> {
        let token_record = self
            .token_manager
            .ensure_active_token(ProviderVariant::Twitch, AccountVariant::Main)
            .await
            .ok()?;

        match &token_record.token {
            crate::token::enums::TokenEnum::Twitch {
                access_token,
                refresh_token,
                ..
            } => {
                let http_client = reqwest::Client::new();
                let token = UserToken::from_existing(
                    &http_client,
                    AccessToken::new(access_token.clone()),
                    refresh_token.clone().map(RefreshToken::new),
                    ClientSecret::new(self.client_secret.clone()),
                )
                .await
                .ok()?;
                self.needs_reauth.store(false, Ordering::Relaxed);
                Some(Arc::new(token))
            }
        }
    }

    pub fn needs_reauth(&self) -> bool {
        self.needs_reauth.load(Ordering::Relaxed)
    }

    pub fn set_needs_reauth(&self, value: bool) {
        self.needs_reauth.store(value, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl OAuthService for TwitchApiClient {
    async fn get_auth_url(&self) -> AppResult<String> {
        self.get_oauth_url().await
    }

    async fn handle_callback(&self, code: &str) -> AppResult<bool> {
        self.exchange_code(code).await
    }
}
