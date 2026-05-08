use chrono::{DateTime, Utc};

use crate::{
    error::AppResult,
    infrastructure::db_sqlite::{SqliteDb, models::TokenRecordRow},
    providers::token_repository::{AccountVariant, ProviderVariant, TokenRecord, TokenRepository},
    token_manager::TokenEnum,
};

#[async_trait::async_trait]
impl TokenRepository for SqliteDb {
    async fn get_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
    ) -> AppResult<Option<TokenEnum>> {
        let result = sqlx::query_as::<_, TokenRecordRow>(
            "SELECT account_variant, provider, raw_data FROM provider_tokens WHERE account_variant = ? AND provider = ?",
        )
        .bind(account_variant)
        .bind(provider)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(token_row) => {
                let record: TokenRecord = token_row.try_into()?;
                Ok(Some(record.token))
            }
            None => Ok(None),
        }
    }

    async fn save_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
        provider_id: &str,
        expires_at: DateTime<Utc>,
        token: TokenEnum,
    ) -> AppResult<()> {
        let raw_data = serde_json::to_string(&token)?;

        sqlx::query(
            "INSERT OR REPLACE INTO provider_tokens (account_variant, provider, provider_id, expires_at, raw_data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(account_variant)
        .bind(provider)
        .bind(provider_id)
        .bind(expires_at)
        .bind(raw_data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
