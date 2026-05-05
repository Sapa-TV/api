use crate::{
    error::AppError,
    providers::token_repository::{ProviderVariant, TokenRecord},
    push::PushSubscription,
};
use sqlx::FromRow;

#[derive(FromRow)]
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

#[derive(FromRow)]
pub struct TokenRecordRow {
    pub account_variant: String,
    pub provider: ProviderVariant,
    pub token: serde_json::Value,
}

impl TryFrom<TokenRecordRow> for TokenRecord {
    type Error = AppError;
    fn try_from(row: TokenRecordRow) -> Result<Self, Self::Error> {
        let token = serde_json::from_value(row.token)?;

        Ok(Self {
            account_variant: row.account_variant,
            provider: row.provider,
            token,
        })
    }
}
