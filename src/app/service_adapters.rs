use std::sync::Arc;

use crate::app::ports::{OAuthService, PushService, SupportersService};
use crate::error::AppResult;
use crate::eventsub::infra::client::TwitchApiClientTrait;
use crate::push::domain::PushSubscription;
use crate::push::domain::PushSubscriptionRepository;
use crate::push::infra::SqlitePushSubscriptionRepository;
use crate::supporters::domain::SupporterRepository;
use crate::supporters::infra::SqliteSupporterRepository;

pub struct TwitchOAuthService {
    client: Arc<dyn TwitchApiClientTrait>,
}

impl TwitchOAuthService {
    pub fn new(client: Arc<dyn TwitchApiClientTrait>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl OAuthService for TwitchOAuthService {
    async fn get_auth_url(&self) -> AppResult<String> {
        self.client.get_oauth_url().await
    }

    async fn handle_callback(&self, code: &str) -> AppResult<bool> {
        self.client.exchange_code(code).await
    }
}

pub struct SqliteSupporterService {
    repo: Arc<SqliteSupporterRepository>,
}

impl SqliteSupporterService {
    pub fn new(repo: Arc<SqliteSupporterRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl SupportersService for SqliteSupporterService {
    async fn get_king(&self) -> AppResult<Option<String>> {
        self.repo.get_king().await
    }

    async fn set_king(&self, name: &str) -> AppResult<()> {
        self.repo.insert_king(name).await
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        self.repo.get_day_supporters().await
    }

    async fn add_day_supporter(&self, name: &str) -> AppResult<()> {
        self.repo.insert_day_supporter(name).await
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        self.repo.get_month_supporters().await
    }

    async fn add_month_supporter(&self, name: &str) -> AppResult<()> {
        self.repo.insert_month_supporter(name).await
    }
}

pub struct SqlitePushService {
    repo: Arc<SqlitePushSubscriptionRepository>,
}

impl SqlitePushService {
    pub fn new(repo: Arc<SqlitePushSubscriptionRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl PushService for SqlitePushService {
    async fn subscribe(
        &self,
        endpoint: &str,
        p256dh: &str,
        auth: &str,
        user_id: Option<&str>,
    ) -> AppResult<()> {
        self.repo
            .insert_subscription(endpoint, p256dh, auth, user_id)
            .await
    }

    async fn unsubscribe(&self, endpoint: &str) -> AppResult<()> {
        self.repo.delete_subscription(endpoint).await
    }

    async fn get_all_subscriptions(&self) -> AppResult<Vec<PushSubscription>> {
        self.repo.get_all_subscriptions().await
    }
}
