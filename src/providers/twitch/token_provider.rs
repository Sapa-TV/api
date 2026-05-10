use reqwest::Client;

use crate::error::{AppError, AppResult};
use crate::token::enums::TokenEnum;
use crate::token::provider::TokenProvider;
use crate::token::types::TokenRecord;

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const TWITCH_VALIDATE_URL: &str = "https://id.twitch.tv/oauth2/validate";

const SCOPES: &[&str] = &[
    "chat:read",
    "chat:edit",
    "channel:read:subscriptions",
    "channel:read:stream_key",
    "user:read:chat",
];

pub struct TwitchTokenProvider {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: Client,
}

impl TwitchTokenProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            http_client: Client::new(),
        }
    }

    pub fn from_env() -> AppResult<Self> {
        let client_id = std::env::var("TWITCH_CLIENT_ID")
            .map_err(|_| AppError::Env("TWITCH_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("TWITCH_CLIENT_SECRET")
            .map_err(|_| AppError::Env("TWITCH_CLIENT_SECRET not set".to_string()))?;
        let redirect_uri = std::env::var("TWITCH_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/api/oauth/callback".to_string());
        Ok(Self::new(client_id, client_secret, redirect_uri))
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    async fn get_user_id(&self, access_token: &str) -> AppResult<String> {
        #[derive(serde::Deserialize)]
        struct ValidateResponse {
            user_id: String,
        }

        let resp = self
            .http_client
            .get(TWITCH_VALIDATE_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Token validation request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Internal("Failed to validate token".to_string()));
        }

        let validate: ValidateResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse validate response: {}", e)))?;

        Ok(validate.user_id)
    }
}

#[async_trait::async_trait]
impl TokenProvider for TwitchTokenProvider {
    fn generate_url(&self) -> AppResult<String> {
        let scopes = SCOPES.join(" ");
        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}",
            TWITCH_AUTH_URL,
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
        );
        Ok(url)
    }

    async fn exchange_token(&self, code: &str) -> AppResult<TokenEnum> {
        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: u64,
        }

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        tracing::info!("Exchanging OAuth code for access token");

        let resp = self
            .http_client
            .post(TWITCH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Token exchange request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Token exchange failed ({}): {}",
                status, body
            )));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

        tracing::debug!(
            "Token response received, expires_in={}",
            token_resp.expires_in
        );

        let user_id = self.get_user_id(&token_resp.access_token).await?;

        let expires_at = chrono::Utc::now().timestamp() + token_resp.expires_in as i64;

        let token_enum = TokenEnum::Twitch {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
            expires_at: Some(expires_at),
            user_id: user_id.clone(),
        };

        tracing::info!(
            "OAuth exchange successful: user_id={}, expires_at={}",
            user_id,
            expires_at
        );

        Ok(token_enum)
    }

    async fn validate_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum> {
        match &token.token {
            TokenEnum::Twitch { expires_at, .. } => {
                let now = chrono::Utc::now().timestamp();
                if let Some(exp) = expires_at {
                    if now < *exp - 300 {
                        tracing::debug!("Token still valid, expires_at={}", exp);
                        return Ok(token.token.clone());
                    }
                    tracing::debug!(
                        "Token expired or near expiry, expires_at={}, now={}",
                        exp,
                        now
                    );
                }
                tracing::info!("Refreshing token (force_refresh_token)");
                self.force_refresh_token(token).await
            }
        }
    }

    async fn force_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum> {
        let refresh_token = match &token.token {
            TokenEnum::Twitch { refresh_token, .. } => refresh_token
                .clone()
                .ok_or_else(|| AppError::Internal("No refresh token available".to_string()))?,
        };

        #[derive(serde::Deserialize)]
        struct RefreshResponse {
            access_token: String,
            #[allow(dead_code)]
            refresh_token: Option<String>,
            expires_in: u64,
        }

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
        ];

        tracing::info!("Refreshing OAuth token");

        let resp = self
            .http_client
            .post(TWITCH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Token refresh request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Token refresh failed ({}): {}",
                status, body
            )));
        }

        let refresh_resp: RefreshResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse refresh response: {}", e)))?;

        let expires_at = chrono::Utc::now().timestamp() + refresh_resp.expires_in as i64;

        let user_id = match &token.token {
            TokenEnum::Twitch { user_id, .. } => user_id.clone(),
        };

        let new_refresh_token = refresh_resp.refresh_token.or(Some(refresh_token));

        let token_enum = TokenEnum::Twitch {
            access_token: refresh_resp.access_token,
            refresh_token: new_refresh_token,
            expires_at: Some(expires_at),
            user_id: user_id.clone(),
        };

        tracing::info!(
            "Token refresh successful: user_id={}, expires_at={}",
            user_id,
            expires_at
        );

        Ok(token_enum)
    }
}
