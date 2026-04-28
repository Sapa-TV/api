use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use twitch_api::HelixClient;

use crate::db::Db;
use crate::error::AppResult;
use crate::twitch::auth::UserTokenManager;
use crate::twitch::eventsub;

pub struct EventSubManager {
    token_manager: Arc<UserTokenManager>,
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    db: Arc<dyn Db + Send + Sync>,
    shutdown_tx: Arc<RwLock<Option<broadcast::Sender<()>>>>,
    handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl EventSubManager {
    fn new(
        token_manager: Arc<UserTokenManager>,
        helix: Arc<HelixClient<'static, reqwest::Client>>,
        db: Arc<dyn Db + Send + Sync>,
    ) -> Self {
        Self {
            token_manager,
            helix,
            db,
            shutdown_tx: Arc::new(RwLock::new(None)),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    async fn start(&self) -> AppResult<()> {
        self.stop().await;

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let token_manager = self.token_manager.clone();
        let helix = self.helix.clone();

        let handle = tokio::spawn(async move {
            tracing::info!("Starting Twitch EventSub listener");
            eventsub::start_eventsub_task(token_manager, helix, shutdown_rx).await;
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
    pub token_manager: Arc<UserTokenManager>,
    pub helix: Arc<HelixClient<'static, reqwest::Client>>,
    eventsub_manager: Arc<EventSubManager>,
}

pub struct AppServicesBuilder {
    db: Option<Arc<dyn Db + Send + Sync>>,
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
    token_manager: Option<Arc<UserTokenManager>>,
    helix: Option<Arc<HelixClient<'static, reqwest::Client>>>,
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
            token_manager: None,
            helix: None,
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

    pub fn token_manager(mut self, token_manager: Arc<UserTokenManager>) -> Self {
        self.token_manager = Some(token_manager);
        self
    }

    pub fn helix(mut self, helix: Arc<HelixClient<'static, reqwest::Client>>) -> Self {
        self.helix = Some(helix);
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

        let token_manager = match self.token_manager {
            Some(tm) => tm,
            None => {
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

                Arc::new(UserTokenManager::new(
                    client_id,
                    client_secret,
                    redirect_uri,
                ))
            }
        };

        token_manager.load_from_db(db.as_ref()).await?;

        let helix = match self.helix {
            Some(h) => h,
            None => Arc::new(HelixClient::new()),
        };

        let eventsub_manager = Arc::new(EventSubManager::new(
            Arc::clone(&token_manager),
            helix.clone(),
            Arc::clone(&db),
        ));

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
            token_manager,
            helix,
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

impl AppServices {
    pub fn builder() -> AppServicesBuilder {
        AppServicesBuilder::new()
    }

    pub async fn restart_eventsub(&self) -> AppResult<()> {
        self.eventsub_manager.restart().await
    }

    fn start_eventsub_watcher(&self) {
        let token_manager = self.token_manager.clone();
        let eventsub_manager = self.eventsub_manager.clone();

        tokio::spawn(async move {
            let mut rx = token_manager.subscribe_token_changes();
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
