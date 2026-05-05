use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct PushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub user_id: Option<String>,
}

#[async_trait::async_trait]
pub trait PushSubscriptionRepository {
    async fn insert_subscription(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()>;
    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>>;
    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()>;
}
