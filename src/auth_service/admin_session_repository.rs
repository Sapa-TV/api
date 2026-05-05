use chrono::{DateTime, Utc};

use crate::error::AppResult;

pub struct AdminSessionInfo {
    pub twitch_id: String,
    pub username: String,
}

#[async_trait::async_trait]
pub trait AdminSessionRepository {
    async fn create_admin_session(
        &self,
        id: &str,
        twitch_id: &str,
        username: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()>;
    async fn get_admin_session(&self, id: &str) -> AppResult<Option<AdminSessionInfo>>;
    async fn delete_admin_session(&self, id: &str) -> AppResult<()>;
    async fn clean_expired_admin_sessions(&self) -> AppResult<()>;
}
