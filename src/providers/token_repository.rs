use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{error::AppResult, providers::twitch::AppTwitchToken};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TokenEnum {
    Twitch(AppTwitchToken),
}

#[derive(Display, EnumString, sqlx::Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum ProviderVariant {
    Twitch,
}

#[derive(Display, EnumString, sqlx::Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum AccountVariant {
    Main,
    Bot,
}

pub struct TokenRecord {
    pub account_variant: String,
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
        account_variant: AccountVariant,
        provider: ProviderVariant,
        provider_id: &str,
        expires_at: DateTime<Utc>,
        token: TokenEnum,
    ) -> AppResult<()>;
}
