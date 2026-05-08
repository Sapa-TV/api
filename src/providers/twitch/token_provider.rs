use axum::http::Request;
use reqwest::Client;
use twitch_api::twitch_oauth2::{
    TwitchToken, UserToken,
    tokens::UserTokenBuilder,
    types::{AccessToken, ClientId, ClientSecret, RefreshToken},
    url::Url,
};

use crate::{
    error::{AppError, AppResult},
    providers::token_repository::TokenRecord,
    token_manager::{TokenEnum, TokenProvider},
};

use super::auth::{TWITCH_SCOPES, TWITCH_SCOPES_VALIDATOR};

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
}

fn user_token_to_enum(token: &UserToken) -> TokenEnum {
    TokenEnum::Twitch {
        access_token: token.access_token.to_string(),
        refresh_token: token.refresh_token.clone().map(|r| r.to_string()),
        expires_at: None,
        user_id: token.user_id.to_string(),
    }
}

#[async_trait::async_trait]
impl TokenProvider for TwitchTokenProvider {
    async fn validate_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum> {
        if let TokenEnum::Twitch { ref user_id, .. } = token.token {
            if !user_id.is_empty() {
                Ok(token.token.clone())
            } else {
                Err(AppError::Unauthorized("Token invalid".to_string()))
            }
        } else {
            Err(AppError::Internal("Invalid token type".to_string()))
        }
    }

    async fn force_refresh_token(&self, token: &TokenRecord) -> AppResult<TokenEnum> {
        if let TokenEnum::Twitch {
            ref access_token,
            ref refresh_token,
            ..
        } = token.token
        {
            let new_token = UserToken::from_existing_or_refresh_token(
                &self.http_client,
                AccessToken::new(access_token.clone()),
                RefreshToken::new(refresh_token.clone().unwrap_or_default()),
                ClientId::new(self.client_id.clone()),
                ClientSecret::new(self.client_secret.clone()),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to refresh token: {}", e)))?;

            Ok(user_token_to_enum(&new_token))
        } else {
            Err(AppError::Internal("Invalid token type".to_string()))
        }
    }

    async fn exchange_token(&self, code: &str) -> AppResult<TokenEnum> {
        tracing::info!("Exchanging authorization code for tokens...");

        let redirect_url = Url::parse(&self.redirect_uri)
            .map_err(|e| AppError::Internal(format!("Invalid redirect URI: {}", e)))?;

        let mut builder = UserTokenBuilder::new(
            self.client_id.clone(),
            self.client_secret.clone(),
            redirect_url,
        );

        builder = builder.set_scopes(TWITCH_SCOPES.to_vec());

        let http_request: Request<Vec<u8>> = builder.get_user_token_request(code);

        let (parts, body) = http_request.into_parts();
        let mut reqwest_request = self
            .http_client
            .request(parts.method.clone(), parts.uri.to_string());

        for (name, value) in parts.headers.iter() {
            reqwest_request = reqwest_request.header(name.as_str(), value.to_str().unwrap_or(""));
        }

        let response = reqwest_request
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to exchange code: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Code exchange failed: {} - {}",
                status, text
            )));
        }

        let twitch_response = response
            .json::<twitch_api::twitch_oauth2::id::TwitchTokenResponse>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

        let user_token = UserToken::from_existing_or_refresh_token(
            &self.http_client,
            twitch_response.access_token,
            twitch_response.refresh_token.unwrap(),
            ClientId::new(self.client_id.clone()),
            ClientSecret::new(self.client_secret.clone()),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create UserToken: {}", e)))?;

        let expected_user_id = std::env::var("TWITCH_USER_ID")
            .map_err(|_| AppError::Env("TWITCH_USER_ID not set".to_string()))?;

        if user_token.user_id.as_ref() != expected_user_id {
            return Err(AppError::Unauthorized("user_id mismatch".to_string()));
        }

        let scopes_valid = TWITCH_SCOPES_VALIDATOR.matches(user_token.scopes());
        if !scopes_valid {
            tracing::warn!("Token scopes from exchange do not match required scopes");
        }

        tracing::info!("OAuth authorization successful");
        Ok(user_token_to_enum(&user_token))
    }

    fn generate_url(&self) -> AppResult<String> {
        let redirect_url = Url::parse(&self.redirect_uri)
            .map_err(|e| AppError::Internal(format!("Invalid redirect URI: {}", e)))?;

        let mut builder = UserTokenBuilder::new(
            self.client_id.clone(),
            self.client_secret.clone(),
            redirect_url,
        );

        builder = builder.set_scopes(TWITCH_SCOPES.to_vec());

        let (url, _csrf) = builder.generate_url();

        Ok(url.to_string())
    }
}
