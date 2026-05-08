use axum::http;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{RwLock, broadcast};
use twitch_api::twitch_oauth2::{
    Scope, TwitchToken, Validator,
    tokens::UserToken,
    tokens::UserTokenBuilder,
    types::{AccessToken, ClientId, ClientSecret, RefreshToken},
    url::Url,
    validator,
};

use crate::{
    error::{AppError, AppResult},
    providers::token_repository::{AccountVariant, ProviderVariant, TokenRepository},
};

macro_rules! define_scopes {
    ($($s:expr),* $(,)?) => {
        pub const TWITCH_SCOPES: &[Scope] = &[ $($s),* ];

        pub const TWITCH_SCOPES_VALIDATOR: Validator = validator!($($s),*);
    };
}

define_scopes![
    Scope::UserReadEmail,
    Scope::ChannelReadSubscriptions,
    Scope::ChannelReadGuestStar,
    Scope::UserReadChat,
    Scope::ChatEdit,
    Scope::ModerationRead,
    Scope::ChannelReadRedemptions,
    Scope::ChannelManageRedemptions,
    Scope::ChannelReadStreamKey,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: String,
}

pub struct UserTokenManager {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: Client,
    token: Arc<RwLock<Option<UserToken>>>,
    pending_csrf: Arc<RwLock<Option<String>>>,
    token_change_tx: broadcast::Sender<()>,
}

// impl UserTokenManager {
//     pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
//         let (token_change_tx, _) = broadcast::channel(1);
//         Self {
//             client_id,
//             client_secret,
//             redirect_uri,
//             http_client: Client::new(),
//             token: Arc::new(RwLock::new(None)),
//             pending_csrf: Arc::new(RwLock::new(None)),
//             token_change_tx,
//         }
//     }

//     pub fn subscribe_token_changes(&self) -> broadcast::Receiver<()> {
//         self.token_change_tx.subscribe()
//     }

//     pub async fn load_from_db<T: TokenRepository>(&self, db: &T) -> AppResult<bool> {
//         let mut scopes_valid = true;
//         if let Some(raw_data) = db
//             .get_provider_token(ProviderVariant::Twitch, AccountVariant::Main)
//             .await?
//         {
//             if let Ok(stored) = serde_json::from_str::<StoredToken>(&raw_data) {
//                 if !stored.refresh_token.is_empty() {
//                     tracing::info!("Loaded Twitch token from database, validating...");

//                     let user_token = UserToken::from_existing_or_refresh_token(
//                         &self.http_client,
//                         AccessToken::new(stored.access_token),
//                         RefreshToken::new(stored.refresh_token),
//                         ClientId::new(self.client_id.clone()),
//                         ClientSecret::new(self.client_secret.clone()),
//                     )
//                     .await
//                     .map_err(|e| {
//                         AppError::Internal(format!("Failed to validate stored token: {}", e))
//                     })?;

//                     let mut token_guard = self.token.write().await;
//                     *token_guard = Some(user_token.clone());

//                     if !TWITCH_SCOPES_VALIDATOR.matches(user_token.scopes()) {
//                         tracing::warn!("Token scopes do not match required scopes");
//                         scopes_valid = false;
//                     }
//                 }
//             }
//         }
//         Ok(scopes_valid)
//     }

//     pub async fn get_access_token(&self) -> Option<String> {
//         let token_guard = self.token.read().await;
//         token_guard.as_ref().map(|t| t.access_token.clone().into())
//     }

//     pub async fn get_token(&self) -> Option<Arc<twitch_api::twitch_oauth2::UserToken>> {
//         let token_guard = self.token.read().await;
//         token_guard.as_ref().map(|t| Arc::new(t.clone()))
//     }

//     pub async fn get_broadcaster_id(&self) -> Option<String> {
//         let token_guard = self.token.read().await;
//         token_guard.as_ref().map(|t| t.user_id.clone().into())
//     }

//     pub async fn get_oauth_url(&self) -> AppResult<String> {
//         let redirect_url = Url::parse(&self.redirect_uri)
//             .map_err(|e| AppError::Internal(format!("Invalid redirect URI: {}", e)))?;

//         let mut builder = UserTokenBuilder::new(
//             self.client_id.clone(),
//             self.client_secret.clone(),
//             redirect_url,
//         );

//         builder = builder.set_scopes(TWITCH_SCOPES.to_vec());

//         let (url, csrf) = builder.generate_url();

//         {
//             let mut pending = self.pending_csrf.write().await;
//             *pending = Some(csrf.to_string());
//         }

//         Ok(url.to_string())
//     }

//     pub async fn exchange_code<T: TokenRepository>(&self, db: &T, code: &str) -> AppResult<bool> {
//         tracing::info!("Exchanging authorization code for tokens...");

//         {
//             let pending = self.pending_csrf.read().await;
//             if pending.is_none() {
//                 return Err(AppError::Internal(
//                     "No CSRF token found. Please generate OAuth URL first.".to_string(),
//                 ));
//             }
//         }

//         let mut builder = UserTokenBuilder::new(
//             self.client_id.clone(),
//             self.client_secret.clone(),
//             Url::parse(&self.redirect_uri)
//                 .map_err(|e| AppError::Internal(format!("Invalid redirect URI: {}", e)))?,
//         );

//         builder = builder.set_scopes(TWITCH_SCOPES.to_vec());

//         let http_request: http::Request<Vec<u8>> = builder.get_user_token_request(code);

//         let (parts, body) = http_request.into_parts();
//         let mut reqwest_request = self
//             .http_client
//             .request(parts.method.clone(), parts.uri.to_string());

//         for (name, value) in parts.headers.iter() {
//             reqwest_request = reqwest_request.header(name.as_str(), value.to_str().unwrap_or(""));
//         }

//         let response = reqwest_request
//             .body(body)
//             .send()
//             .await
//             .map_err(|e| AppError::Internal(format!("Failed to exchange code: {}", e)))?;

//         if !response.status().is_success() {
//             let status = response.status();
//             let text = response.text().await.unwrap_or_default();
//             return Err(AppError::Internal(format!(
//                 "Code exchange failed: {} - {}",
//                 status, text
//             )));
//         }

//         let twitch_response = response
//             .json::<twitch_api::twitch_oauth2::id::TwitchTokenResponse>()
//             .await
//             .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

//         {
//             let mut pending = self.pending_csrf.write().await;
//             *pending = None;
//         }

//         let user_token = UserToken::from_existing_or_refresh_token(
//             &self.http_client,
//             twitch_response.access_token,
//             twitch_response.refresh_token.unwrap(),
//             ClientId::new(self.client_id.clone()),
//             ClientSecret::new(self.client_secret.clone()),
//         )
//         .await
//         .map_err(|e| AppError::Internal(format!("Failed to create UserToken: {}", e)))?;

//         let expected_user_id = std::env::var("TWITCH_USER_ID")
//             .map_err(|_| AppError::Env("TWITCH_USER_ID not set".to_string()))?;

//         if user_token.user_id.as_ref() != expected_user_id {
//             return Err(AppError::Unauthorized("user_id mismatch".to_string()));
//         }

//         let stored_token = StoredToken {
//             access_token: user_token.access_token.clone().into(),
//             refresh_token: user_token
//                 .refresh_token
//                 .clone()
//                 .map(|r| r.into())
//                 .unwrap_or_default(),
//             created_at: format!("{:?}", SystemTime::now()),
//         };

//         let raw_data = serde_json::to_string(&stored_token)
//             .map_err(|e| AppError::Internal(format!("Failed to serialize token: {}", e)))?;

//         let expires_at = Utc::now() + user_token.expires_in();

//         db.save_provider_token(
//             AccountVariant::Main,
//             ProviderVariant::Twitch,
//             user_token.user_id.as_str(),
//             expires_at,
//             &raw_data,
//         )
//         .await?;

//         {
//             let mut token_guard = self.token.write().await;
//             *token_guard = Some(user_token.clone());
//         }

//         let scopes_valid = TWITCH_SCOPES_VALIDATOR.matches(user_token.scopes());
//         if !scopes_valid {
//             tracing::warn!("Token scopes from exchange do not match required scopes");
//         }

//         let _ = self.token_change_tx.send(());

//         tracing::info!("OAuth authorization successful, token saved to database");
//         Ok(scopes_valid)
//     }
// }
