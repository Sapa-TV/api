use crate::{
    auth_service::AdminWhiteListRepository, error::AppResult, infrastructure::db_sqlite::SqliteDb,
};

// TODO: refactor to use provider enum

#[async_trait::async_trait]
impl AdminWhiteListRepository for SqliteDb {
    async fn is_admin_whitelisted(&self, twitch_id: &str) -> AppResult<bool> {
        let result: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM admin_whitelist WHERE provider = 'twitch' AND provider_id = ?",
        )
        .bind(twitch_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(result.is_some())
    }
    async fn add_admin_to_whitelist(
        &self,
        twitch_id: &str,
        username: Option<&str>,
        added_by: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO admin_whitelist (twitch_id, username, added_by) VALUES (?, ?, ?)",
        )
        .bind(twitch_id)
        .bind(username)
        .bind(added_by)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn remove_admin_from_whitelist(&self, twitch_id: &str) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM admin_whitelist WHERE provider = 'twitch' AND provider_id = ?",
        )
        .bind(twitch_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
    async fn list_admin_whitelist(&self) -> AppResult<Vec<(String, Option<String>)>> {
        let results: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT provider_id, username FROM admin_whitelist WHERE provider = 'twitch'",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(results)
    }
}
