use crate::error::AppResult;
use crate::token::enums::TokenEnum;
use crate::token::types::{AccountVariant, ProviderVariant};
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait TokenRepository: Send + Sync {
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
