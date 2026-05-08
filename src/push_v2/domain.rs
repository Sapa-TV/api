use crate::error::AppResult;

pub use crate::push::PushSubscription;

#[async_trait::async_trait]
pub trait PushSubscriptionRepository: Send + Sync {
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
