use chrono::{DateTime, Utc};
use sqlx::prelude::Type;

use crate::{error::AppResult, token_manager::TokenEnum};

#[derive(Type, Hash, PartialEq, Eq, Clone)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum ProviderVariant {
    Twitch,
}

#[derive(Type, Hash, PartialEq, Eq, Clone)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum AccountVariant {
    Main,
    Bot,
}

#[derive(Clone)]
pub struct TokenRecord {
    pub account_variant: AccountVariant,
    pub provider: ProviderVariant,
    pub token: TokenEnum,
}

#[async_trait::async_trait]
pub trait TokenRepository {
    async fn get_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
    ) -> AppResult<Option<TokenEnum>>;
    async fn save_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
        provider_id: &str,
        expires_at: DateTime<Utc>,
        token: TokenEnum,
    ) -> AppResult<()>;
}
