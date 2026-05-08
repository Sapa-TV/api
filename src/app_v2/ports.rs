use crate::push_v2::domain::PushSubscription;

#[async_trait::async_trait]
pub trait SupportersService: Send + Sync {
    async fn get_king(&self) -> crate::error::AppResult<Option<String>>;
    async fn set_king(&self, name: &str) -> crate::error::AppResult<()>;
    async fn get_day_supporters(&self) -> crate::error::AppResult<Vec<String>>;
    async fn add_day_supporter(&self, name: &str) -> crate::error::AppResult<()>;
    async fn get_month_supporters(&self) -> crate::error::AppResult<Vec<String>>;
    async fn add_month_supporter(&self, name: &str) -> crate::error::AppResult<()>;
}

#[async_trait::async_trait]
pub trait PushService: Send + Sync {
    async fn subscribe(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> crate::error::AppResult<()>;
    async fn unsubscribe(&self, endpoint: &str) -> crate::error::AppResult<()>;
    async fn get_all_subscriptions(&self) -> crate::error::AppResult<Vec<PushSubscription>>;
}

#[async_trait::async_trait]
pub trait OAuthService: Send + Sync {
    async fn get_auth_url(&self) -> crate::error::AppResult<String>;
    async fn handle_callback(&self, code: &str) -> crate::error::AppResult<bool>;
}
