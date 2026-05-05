use chrono::{DateTime, Utc};

use crate::{
    auth_service::{AdminSessionInfo, AdminSessionRepository},
    error::AppResult,
    infrastructure::db_sqlite::SqliteDb,
};

#[async_trait::async_trait]
impl AdminSessionRepository for SqliteDb {
    async fn create_admin_session(
        &self,
        id: &str,
        twitch_id: &str,
        username: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO admin_sessions (id, provider, provider_id, username, expires_at) VALUES (?, 'twitch', ?, ?, ?)",
        )
        .bind(id)
        .bind(twitch_id)
        .bind(username)
        .bind(expires_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_admin_session(&self, id: &str) -> AppResult<Option<AdminSessionInfo>> {
        let result: Option<(String, String)> = sqlx::query_as(
            "SELECT provider_id, username FROM admin_sessions WHERE id = ? AND expires_at > datetime('now')",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.map(|(twitch_id, username)| AdminSessionInfo {
            twitch_id,
            username,
        }))
    }

    async fn delete_admin_session(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM admin_sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn clean_expired_admin_sessions(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM admin_sessions WHERE expires_at <= datetime('now')")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
