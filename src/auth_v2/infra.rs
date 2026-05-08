use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::auth_v2::domain::{AdminSessionInfo, AdminSessionRepository};
use crate::error::AppResult;
use crate::shared_infra_v2::sqlite_db::SqliteDb;

#[derive(Clone)]
pub struct SqliteAdminSessionRepository {
    db: Arc<SqliteDb>,
}

impl SqliteAdminSessionRepository {
    pub fn new(db: Arc<SqliteDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl AdminSessionRepository for SqliteAdminSessionRepository {
    async fn create_admin_session(
        &self,
        id: &str,
        provider: &str,
        provider_id: &str,
        username: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO admin_sessions (id, provider, provider_id, username, expires_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(provider)
        .bind(provider_id)
        .bind(username)
        .bind(expires_at.to_rfc3339())
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    async fn get_admin_session(&self, id: &str) -> AppResult<Option<AdminSessionInfo>> {
        let result: Option<(String, String, String)> = sqlx::query_as(
            "SELECT provider, provider_id, username FROM admin_sessions WHERE id = ? AND expires_at > datetime('now')",
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;
        Ok(result.map(|(provider, provider_id, username)| AdminSessionInfo {
            provider,
            provider_id,
            username,
        }))
    }

    async fn delete_admin_session(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM admin_sessions WHERE id = ?")
            .bind(id)
            .execute(self.db.pool())
            .await?;
        Ok(())
    }

    async fn clean_expired_admin_sessions(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM admin_sessions WHERE expires_at <= datetime('now')")
            .execute(self.db.pool())
            .await?;
        Ok(())
    }
}
