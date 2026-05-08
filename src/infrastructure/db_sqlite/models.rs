use crate::{
    error::AppError,
    providers::token_repository::{AccountVariant, ProviderVariant},
    push::PushSubscription,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushSubscriptionRow {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_id: Option<String>,
}

impl From<PushSubscriptionRow> for PushSubscription {
    fn from(row: PushSubscriptionRow) -> Self {
        Self {
            endpoint: row.endpoint,
            p256dh: row.p256dh,
            auth: row.auth,
            user_id: row.user_id,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TokenRecordRow {
    pub account_variant: String,
    pub provider: String,
    pub raw_data: String,
}

impl TryFrom<TokenRecordRow> for crate::providers::token_repository::TokenRecord {
    type Error = AppError;
    fn try_from(row: TokenRecordRow) -> Result<Self, Self::Error> {
        let account_variant = match row.account_variant.as_str() {
            "main" => AccountVariant::Main,
            "bot" => AccountVariant::Bot,
            _ => {
                return Err(AppError::Internal(format!(
                    "Unknown account_variant: {}",
                    row.account_variant
                )));
            }
        };
        let provider = match row.provider.as_str() {
            "twitch" => ProviderVariant::Twitch,
            _ => {
                return Err(AppError::Internal(format!(
                    "Unknown provider: {}",
                    row.provider
                )));
            }
        };
        let token = serde_json::from_str(&row.raw_data)?;

        Ok(Self {
            account_variant,
            provider,
            token,
        })
    }
}
