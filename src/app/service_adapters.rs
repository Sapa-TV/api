use std::sync::Arc;

use crate::app::ports::{PushService, SupportersService};
use crate::error::AppResult;
use crate::push::subscription::PushSubscription;
use crate::push::subscription::PushSubscriptionRepository;
use crate::state::in_memory_repository::InMemoryStateRepository;
use crate::state::repository::StateRepository;
use crate::supporters::repository::SupporterRepository;

pub struct CachedSupportersService {
    cache: Arc<InMemoryStateRepository>,
    repo: Arc<dyn SupporterRepository>,
}

// TODO: move to supporters module
impl CachedSupportersService {
    // TODO: check is cache fulfilled at start?
    pub fn new(cache: Arc<InMemoryStateRepository>, repo: Arc<dyn SupporterRepository>) -> Self {
        Self { cache, repo }
    }
}

#[async_trait::async_trait]
impl SupportersService for CachedSupportersService {
    async fn get_king(&self) -> AppResult<Option<String>> {
        let cached = self.cache.get_king().await?;
        if cached.is_some() {
            return Ok(cached);
        }
        let from_db = self.repo.get_king().await?;
        if let Some(ref king) = from_db {
            self.cache.set_king(king).await?;
        }
        Ok(from_db)
    }

    async fn set_king(&self, name: &str) -> AppResult<()> {
        self.repo.set_king(name).await?;
        self.cache.set_king(name).await?;
        Ok(())
    }

    async fn get_day_supporters(&self) -> AppResult<Vec<String>> {
        let cached = self.cache.get_day_supporters().await?;
        // TODO: cache CAN be empty, but we should check if it's fulfilled
        if !cached.is_empty() {
            return Ok(cached);
        }
        let from_db = self.repo.get_day_supporters().await?;
        for supporter in &from_db {
            self.cache.add_day_supporter(supporter).await?;
        }
        Ok(from_db)
    }

    async fn add_day_supporter(&self, name: &str) -> AppResult<()> {
        self.repo.insert_day_supporter(name).await?;
        self.cache.add_day_supporter(name).await?;
        Ok(())
    }

    async fn get_month_supporters(&self) -> AppResult<Vec<String>> {
        let cached = self.cache.get_month_supporters().await?;
        // TODO: cache CAN be empty, but we should check if it's fulfilled
        if !cached.is_empty() {
            return Ok(cached);
        }
        let from_db = self.repo.get_month_supporters().await?;
        for supporter in &from_db {
            self.cache.add_month_supporter(supporter).await?;
        }
        Ok(from_db)
    }

    async fn add_month_supporter(&self, name: &str) -> AppResult<()> {
        self.repo.insert_month_supporter(name).await?;
        self.cache.add_month_supporter(name).await?;
        Ok(())
    }
}

pub struct SqlitePushService {
    repo: Arc<dyn PushSubscriptionRepository>,
}

impl SqlitePushService {
    pub fn new(repo: Arc<dyn PushSubscriptionRepository>) -> Self {
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
