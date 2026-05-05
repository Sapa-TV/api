use crate::{
    error::AppResult,
    infrastructure::db_sqlite::{SqliteDb, models::PushSubscriptionRow},
    push::{PushSubscription, PushSubscriptionRepository},
};

#[async_trait::async_trait]
impl PushSubscriptionRepository for SqliteDb {
    async fn insert_subscription(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO push_subscriptions (endpoint, p256dh, auth, user_id) VALUES (?, ?, ?, ?)",
        )
        .bind(endpoint)
        .bind(p256dh)
        .bind(auth)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>> {
        let results = sqlx::query_as::<_, PushSubscriptionRow>(
            "SELECT id, endpoint, p256dh, auth, user_id, created_at FROM push_subscriptions",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(results.into_iter().map(|r| r.into()).collect::<Vec<_>>())
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
