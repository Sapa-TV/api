use chrono::{DateTime, Utc};

use crate::error::AppResult;

pub struct AdminSessionInfo {
    pub provider: String,
    pub provider_id: String,
    pub username: String,
}

#[async_trait::async_trait]
pub trait AdminSessionRepository: Send + Sync {
    async fn create_admin_session(
        &self,
        id: &str,
        provider: &str,
        provider_id: &str,
        username: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()>;
    async fn get_admin_session(&self, id: &str) -> AppResult<Option<AdminSessionInfo>>;
    async fn delete_admin_session(&self, id: &str) -> AppResult<()>;
    async fn clean_expired_admin_sessions(&self) -> AppResult<()>;
}

#[async_trait::async_trait]
pub trait AdminWhiteListRepository: Send + Sync {
    async fn is_admin_whitelisted(&self, provider: &str, provider_id: &str) -> AppResult<bool>;
    async fn add_admin_to_whitelist(
        &self,
        provider: &str,
        provider_id: &str,
        username: Option<&str>,
        added_by: Option<&str>,
    ) -> AppResult<()>;
    async fn remove_admin_from_whitelist(
        &self,
        provider: &str,
        provider_id: &str,
    ) -> AppResult<bool>;
    async fn list_admin_whitelist(&self) -> AppResult<Vec<(String, Option<String>)>>;
}