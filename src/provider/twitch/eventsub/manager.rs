use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use crate::error::AppResult;
use crate::provider::twitch::eventsub::traits::{ChatHandler, StreamLifecycle};

pub struct TwitchLifecycle {
    stream_lifecycle: Arc<dyn StreamLifecycle>,
    chat_handler: Arc<dyn ChatHandler>,
}

impl TwitchLifecycle {
    pub fn new(
        stream_lifecycle: Arc<dyn StreamLifecycle>,
        chat_handler: Arc<dyn ChatHandler>,
    ) -> Self {
        Self {
            stream_lifecycle,
            chat_handler,
        }
    }

    pub async fn on_stream_started(&self, started_at: DateTime<Utc>) -> AppResult<()> {
        self.stream_lifecycle.on_started("twitch", started_at).await
    }

    pub async fn on_stream_ended(&self, ended_at: DateTime<Utc>) -> AppResult<()> {
        self.stream_lifecycle.on_ended("twitch", ended_at).await
    }

    pub async fn on_chat_message(
        &self,
        user_id: &str,
        username: &str,
        message: &str,
        timestamp: DateTime<Utc>,
    ) -> AppResult<()> {
        self.chat_handler
            .on_message("twitch", user_id, username, message, timestamp)
            .await
    }
}

pub struct EventSubManager {
    api_client: Arc<crate::provider::twitch::api::TwitchApiClient>,
    lifecycle: Arc<TwitchLifecycle>,
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
    handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl EventSubManager {
    pub fn new(
        api_client: Arc<crate::provider::twitch::api::TwitchApiClient>,
        lifecycle: Arc<TwitchLifecycle>,
    ) -> Self {
        Self {
            api_client,
            lifecycle,
            shutdown_tx: Arc::new(RwLock::new(None)),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> AppResult<()> {
        self.stop().await;

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let api_client = self.api_client.clone();
        let lifecycle = self.lifecycle.clone();

        let handle = tokio::spawn(async move {
            tracing::info!("Starting Twitch EventSub listener");
            crate::provider::twitch::eventsub::listener::start_eventsub_task(
                api_client,
                lifecycle,
                shutdown_rx,
            )
            .await;
        });

        {
            let mut tx = self.shutdown_tx.write().await;
            *tx = Some(shutdown_tx);
        }
        {
            let mut h = self.handle.write().await;
            *h = Some(handle);
        }

        Ok(())
    }

    pub async fn stop(&self) {
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.write().await.take() {
            handle.abort();
        }
    }
}

pub struct TwitchStreamLifecycleAdapter;

#[async_trait::async_trait]
impl StreamLifecycle for TwitchStreamLifecycleAdapter {
    async fn on_started(&self, provider: &str, started_at: DateTime<Utc>) -> AppResult<()> {
        tracing::info!("[{}] Stream started at {}", provider, started_at);
        Ok(())
    }

    async fn on_ended(&self, provider: &str, ended_at: DateTime<Utc>) -> AppResult<()> {
        tracing::info!("[{}] Stream ended at {}", provider, ended_at);
        Ok(())
    }
}

pub struct TwitchChatHandlerAdapter;

#[async_trait::async_trait]
impl ChatHandler for TwitchChatHandlerAdapter {
    async fn on_message(
        &self,
        provider: &str,
        user_id: &str,
        username: &str,
        message: &str,
        timestamp: DateTime<Utc>,
    ) -> AppResult<()> {
        tracing::info!(
            "[{}] Chat message from {} ({}): {} at {}",
            provider,
            username,
            user_id,
            message,
            timestamp
        );
        Ok(())
    }
}
