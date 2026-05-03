use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

use crate::app_logic::{ChatHandler, StreamLifecycle};
use crate::db::Db;
use crate::error::AppResult;
use crate::providers::twitch::auth::UserTokenManager;
use crate::providers::twitch::client::TwitchApiClient;
use crate::providers::twitch::eventsub;
use crate::providers::twitch::lifecycle::TwitchLifecycle;
use twitch_api::HelixClient;

pub struct EventSubManager {
    api_client: Arc<TwitchApiClient>,
    lifecycle: Arc<TwitchLifecycle>,
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
    handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl EventSubManager {
    fn new(api_client: Arc<TwitchApiClient>, lifecycle: Arc<TwitchLifecycle>) -> Self {
        Self {
            api_client,
            lifecycle,
            shutdown_tx: Arc::new(RwLock::new(None)),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    async fn start(&self) -> AppResult<()> {
        self.stop().await;

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let api_client = self.api_client.clone();
        let lifecycle = self.lifecycle.clone();

        let handle = tokio::spawn(async move {
            tracing::info!("Starting Twitch EventSub listener");
            eventsub::start_eventsub_task(api_client, lifecycle, shutdown_rx).await;
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

    async fn stop(&self) {
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.write().await.take() {
            let _ = handle.await;
        }
    }

    async fn restart(&self) -> AppResult<()> {
        tracing::info!("Restarting EventSub...");
        self.start().await
    }
}

#[derive(Clone)]
pub struct AppServices {
    pub db: Arc<dyn Db + Send + Sync>,
    pub twitch_api: Arc<TwitchApiClient>,
    eventsub_manager: Arc<EventSubManager>,
}

pub struct AppServicesBuilder {
    db: Option<Arc<dyn Db + Send + Sync>>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
    eventsub_enabled: bool,
    watch_token_changes: bool,
}

impl AppServicesBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            client_id: None,
            client_secret: None,
            redirect_uri: None,
            eventsub_enabled: true,
            watch_token_changes: true,
        }
    }

    pub fn db(mut self, db: Arc<dyn Db + Send + Sync>) -> Self {
        self.db = Some(db);
        self
    }

    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn client_secret(mut self, client_secret: String) -> Self {
        self.client_secret = Some(client_secret);
        self
    }

    pub fn redirect_uri(mut self, redirect_uri: String) -> Self {
        self.redirect_uri = Some(redirect_uri);
        self
    }

    pub fn eventsub_enabled(mut self, enabled: bool) -> Self {
        self.eventsub_enabled = enabled;
        self
    }

    pub fn watch_token_changes(mut self, enabled: bool) -> Self {
        self.watch_token_changes = enabled;
        self
    }

    pub async fn build(self) -> AppResult<AppServices> {
        let db = self
            .db
            .ok_or_else(|| crate::error::AppError::Internal("db is required".to_string()))?;

        let client_id = self.client_id.unwrap_or_else(|| {
            std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set")
        });
        let client_secret = self.client_secret.unwrap_or_else(|| {
            std::env::var("TWITCH_CLIENT_SECRET").expect("TWITCH_CLIENT_SECRET not set")
        });
        let redirect_uri = self.redirect_uri.unwrap_or_else(|| {
            std::env::var("TWITCH_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/api/oauth/callback".to_string())
        });

        let token_manager = Arc::new(UserTokenManager::new(
            client_id,
            client_secret,
            redirect_uri,
        ));

        token_manager.load_from_db(db.as_ref()).await?;

        let helix = Arc::new(HelixClient::new());

        let twitch_api = Arc::new(TwitchApiClient::new(helix, Arc::clone(&token_manager)));

        let stream_lifecycle: Arc<dyn StreamLifecycle> = Arc::new(TwitchStreamLifecycleAdapter);
        let chat_handler: Arc<dyn ChatHandler> = Arc::new(TwitchChatHandlerAdapter);

        let lifecycle = Arc::new(TwitchLifecycle::new(stream_lifecycle, chat_handler));

        let eventsub_manager = Arc::new(EventSubManager::new(twitch_api.clone(), lifecycle));

        if self.eventsub_enabled && token_manager.get_access_token().await.is_some() {
            tracing::info!("Twitch user token found, starting EventSub...");
            eventsub_manager.start().await?;
        } else if !self.eventsub_enabled {
            tracing::info!("EventSub disabled by config");
        } else {
            tracing::warn!("No Twitch user token found. EventSub not started.");
        }

        let services = AppServices {
            db,
            twitch_api,
            eventsub_manager,
        };

        if self.watch_token_changes {
            services.start_eventsub_watcher();
        }

        Ok(services)
    }
}

impl Default for AppServicesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct TwitchStreamLifecycleAdapter;

#[async_trait::async_trait]
impl StreamLifecycle for TwitchStreamLifecycleAdapter {
    async fn on_started(
        &self,
        provider: &str,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        tracing::info!("[{}] Stream started at {}", provider, started_at);
        Ok(())
    }

    async fn on_ended(
        &self,
        provider: &str,
        ended_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        tracing::info!("[{}] Stream ended at {}", provider, ended_at);
        Ok(())
    }
}

struct TwitchChatHandlerAdapter;

#[async_trait::async_trait]
impl ChatHandler for TwitchChatHandlerAdapter {
    async fn on_message(
        &self,
        provider: &str,
        user_id: &str,
        username: &str,
        message: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
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

impl AppServices {
    pub fn builder() -> AppServicesBuilder {
        AppServicesBuilder::new()
    }

    pub async fn restart_eventsub(&self) -> AppResult<()> {
        self.eventsub_manager.restart().await
    }

    fn start_eventsub_watcher(&self) {
        let mut rx = self.twitch_api.subscribe_token_changes();
        let eventsub_manager = self.eventsub_manager.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        tracing::info!("Token changed, restarting EventSub...");
                        if let Err(e) = eventsub_manager.restart().await {
                            tracing::error!("Failed to restart EventSub: {}", e);
                        }
                    }
                }
            }
        });
    }
}
