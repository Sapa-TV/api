pub use crate::oauth::service::OAuthService;
use crate::{error::AppResult, push::subscription::PushSubscription};

#[async_trait::async_trait]
pub trait SupportersService: Send + Sync {
    async fn get_king(&self) -> AppResult<Option<String>>;
    async fn set_king(&self, name: &str) -> AppResult<()>;
    async fn get_day_supporters(&self) -> AppResult<Vec<String>>;
    async fn add_day_supporter(&self, name: &str) -> AppResult<()>;
    async fn get_month_supporters(&self) -> AppResult<Vec<String>>;
    async fn add_month_supporter(&self, name: &str) -> AppResult<()>;
}

#[async_trait::async_trait]
pub trait PushService: Send + Sync {
    async fn subscribe(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()>;
    async fn unsubscribe(&self, endpoint: &str) -> AppResult<()>;
    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>>;
}
