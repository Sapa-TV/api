use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::AppResult;

#[async_trait]
pub trait TwitchApiClientTrait: Send + Sync {
    async fn get_oauth_url(&self) -> AppResult<String>;
    async fn exchange_code(&self, code: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait StreamLifecycle: Send + Sync {
    async fn on_started(&self, provider: &str, started_at: DateTime<Utc>) -> AppResult<()>;
    async fn on_ended(&self, provider: &str, ended_at: DateTime<Utc>) -> AppResult<()>;
}

#[async_trait]
pub trait ChatHandler: Send + Sync {
    async fn on_message(
        &self,
        provider: &str,
        user_id: &str,
        username: &str,
        message: &str,
        timestamp: DateTime<Utc>,
    ) -> AppResult<()>;
}
