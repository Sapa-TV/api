use crate::{
    error::AppResult,
    infrastructure::db_sqlite::{SqliteDb, models::TokenRecordRow},
    providers::token_repository::{AccountVariant, ProviderVariant, TokenRecord, TokenRepository},
};
use serde::{Serialize, de::DeserializeOwned};

#[async_trait::async_trait]
impl<T: Serialize + DeserializeOwned + Send + 'static> TokenRepository<T> for SqliteDb {
    async fn get_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
    ) -> AppResult<Option<T>> {
        let result = sqlx::query_as::<_, TokenRecordRow>(
            "SELECT raw_data FROM provider_tokens WHERE account_variant = ? AND provider = ?",
        )
        .bind(account_variant)
        .bind(provider)
        .fetch_optional(&self.pool)
        .await?;

        let result = match result {
            Some(token_row) => {
                let record: TokenRecord<T> = token_row.try_into()?;
                Some(record.token)
            }
            None => None,
        };
        Ok(result)
    }

    async fn save_provider_token(
        &self,
        account_variant: AccountVariant,
        provider: ProviderVariant,
        provider_id: String,
        expires_at: String,
        token: T,
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
