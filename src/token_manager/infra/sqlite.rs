use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::token_manager::domain::enums::TokenEnum;
use crate::token_manager::domain::types::{AccountVariant, ProviderVariant};
use crate::token_manager::domain::{TokenRecord, TokenRepository};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct TokenRecordRow {
    pub account_variant: String,
    pub provider: String,
    pub raw_data: String,
}

impl TryFrom<TokenRecordRow> for TokenRecord {
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

#[derive(Clone)]
pub struct SqliteTokenRepository {
    db: SqlitePool,
}

impl SqliteTokenRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TokenRepository for SqliteTokenRepository {
    async fn get_provider_token(
        &self,
        provider: ProviderVariant,
        account_variant: AccountVariant,
    ) -> AppResult<Option<TokenEnum>> {
        let result = sqlx::query_as::<_, TokenRecordRow>(
            "SELECT account_variant, provider, raw_data FROM provider_tokens WHERE account_variant = ? AND provider = ?",
        )
        .bind(&account_variant)
        .bind(&provider)
        .fetch_optional(&self.db)
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
        .bind(&account_variant)
        .bind(&provider)
        .bind(provider_id)
        .bind(expires_at)
        .bind(raw_data)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}
