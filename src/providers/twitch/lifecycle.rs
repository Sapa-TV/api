use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::app_logic::{ChatHandler, StreamLifecycle};
use crate::error::AppResult;

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

#[async_trait]
impl StreamLifecycle for TwitchLifecycle {
    async fn on_started(&self, provider: &str, started_at: DateTime<Utc>) -> AppResult<()> {
        tracing::info!("[{}] Stream started at {}", provider, started_at);
        Ok(())
    }

    async fn on_ended(&self, provider: &str, ended_at: DateTime<Utc>) -> AppResult<()> {
        tracing::info!("[{}] Stream ended at {}", provider, ended_at);
        Ok(())
    }
}

#[async_trait]
impl ChatHandler for TwitchLifecycle {
    async fn on_message(
        &self,
        provider: &str,
        user_id: &str,
        username: &str,
        message: &str,
        _timestamp: DateTime<Utc>,
    ) -> AppResult<()> {
        tracing::info!(
            "[{}] Chat message from {} ({}): {}",
            provider,
            username,
            user_id,
            message
        );
        Ok(())
    }
}
