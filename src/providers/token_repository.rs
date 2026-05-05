use serde::{Serialize, de::DeserializeOwned};
use strum::{Display, EnumString};

use crate::error::AppResult;

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

pub struct TokenRecord<T: Serialize + DeserializeOwned + Send> {
    pub account_variant: String,
    pub provider: ProviderVariant,
    pub token: T,
}

#[async_trait::async_trait]
pub trait TokenRepository<T: Serialize + DeserializeOwned + Send> {
    async fn get_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
    ) -> AppResult<Option<T>>;
    async fn save_provider_token(
        &self,
        account_variant: AccountVariant,
        provider: ProviderVariant,
        provider_id: String,
        expires_at: String,
        token: T,
    ) -> AppResult<()>;
}
