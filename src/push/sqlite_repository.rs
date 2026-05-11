use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::push::subscription::{PushSubscription, PushSubscriptionRepository};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushSubscriptionRow {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_id: Option<String>,
}

impl From<PushSubscriptionRow> for PushSubscription {
    fn from(row: PushSubscriptionRow) -> Self {
        Self {
            endpoint: row.endpoint,
            p256dh: row.p256dh,
            auth: row.auth,
            user_id: row.user_id,
        }
    }
}

#[derive(Clone)]
pub struct SqlitePushSubscriptionRepository {
    db: SqlitePool,
}

impl SqlitePushSubscriptionRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl PushSubscriptionRepository for SqlitePushSubscriptionRepository {
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
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>> {
        let results = sqlx::query_as::<_, PushSubscriptionRow>(
            "SELECT endpoint, p256dh, auth, user_id FROM push_subscriptions",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(results.into_iter().map(|r| r.into()).collect::<Vec<_>>())
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
            .bind(endpoint)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
