mod database;

mod error;
mod push;
mod supporters;

mod app;
mod health;
mod oauth;
mod provider;
mod router;
mod state;
mod token;

use crate::provider::twitch::api::TwitchApiClient;
use crate::provider::twitch::eventsub::manager::EventSubManager;
use crate::provider::twitch::token_provider::TwitchTokenProvider;
use crate::state::in_memory_repository::InMemoryStateRepository;
use app::app::App;
use app::service_adapters::{CachedSupportersService, SqlitePushService};
use dotenvy::dotenv;
use error::AppResult;
use router::router;
use rustls::crypto::CryptoProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::token::manager::TokenManager;
use crate::token::sqlite_repository::SqliteTokenRepository;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();

    CryptoProvider::install_default(rustls::crypto::ring::default_provider())
        .expect("Failed to install crypto provider");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting application");
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    println!("Commit: {}", env!("GIT_HASH"));

    let db = crate::database::sqlite_pool::create_db_pool().await?;

    let supporters_repo =
        Arc::new(crate::supporters::sqlite_repository::SqliteSupporterRepository::new(db.clone()));
    let push_repo =
        Arc::new(crate::push::sqlite_repository::SqlitePushSubscriptionRepository::new(db.clone()));
    let token_repo = Arc::new(SqliteTokenRepository::new(db.clone()));
    let token_manager = Arc::new(TokenManager::new(token_repo));

    let twitch_provider = Arc::new(TwitchTokenProvider::from_env()?);
    token_manager
        .register_provider(
            crate::token::types::ProviderVariant::Twitch,
            twitch_provider.clone(),
        )
        .await;

    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    let state_cache = Arc::new(InMemoryStateRepository::new());
    let cached_supporters = Arc::new(CachedSupportersService::new(
        state_cache,
        supporters_repo.clone(),
    ));

    let push_client = match crate::push::web_push_client::WebPushClient::from_env() {
        Some(c) => Arc::new(c),
        None => {
            tracing::warn!("WebPushClient not initialized - VAPID keys not configured");
            Arc::new(
                crate::push::web_push_client::WebPushClient::new(
                    "placeholder",
                    "mailto:placeholder",
                )
                .expect("hardcoded"),
            )
        }
    };

    let twitch_api_client = Arc::new(TwitchApiClient::new(
        Arc::new(twitch_api::HelixClient::new()),
        token_manager.clone(),
        twitch_provider.client_secret().to_string(),
    ));

    let lifecycle = Arc::new(
        crate::provider::twitch::eventsub::manager::TwitchLifecycle::new(
            Arc::new(crate::provider::twitch::eventsub::manager::TwitchStreamLifecycleAdapter),
            Arc::new(crate::provider::twitch::eventsub::manager::TwitchChatHandlerAdapter),
        ),
    );
    let eventsub_manager = Arc::new(EventSubManager::new(twitch_api_client.clone(), lifecycle));

    let has_main_token = token_manager
        .get_token(
            crate::token::types::ProviderVariant::Twitch,
            crate::token::types::AccountVariant::Main,
        )
        .await
        .is_ok();
    if has_main_token {
        tracing::info!("Main token found, starting EventSub...");
        eventsub_manager.start().await?;
    } else {
        tracing::info!("No main token found, EventSub will start on token update");
    }

    let eventsub_for_token_listen = eventsub_manager.clone();
    let mut token_change_rx = token_manager.subscribe_token_changes();
    let mut shutdown_rx = shutdown_rx;

    let token_listener = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Token change listener shutting down");
                    break;
                }
                _ = token_change_rx.recv() => {
                    tracing::info!("Token changed, restarting EventSub...");
                    if let Err(e) = eventsub_for_token_listen.start().await {
                        tracing::error!("Failed to restart EventSub: {}", e);
                    }
                }
            }
        }
    });

    let app: App = App::builder()
        .supporters(cached_supporters)
        .push(Arc::new(SqlitePushService::new(push_repo)))
        .oauth(twitch_api_client)
        .token_manager(token_manager)
        .push_client(push_client)
        .eventsub(eventsub_manager.clone())
        .build();

    let app = Arc::new(app);
    let app_for_shutdown = app.clone();
    let app_router = router(app.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Backend API: http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/docs");
    tracing::info!("ReDoc: http://localhost:3000/redoc");
    tracing::info!("OpenAPI JSON: http://localhost:3000/openapi.json");
    tracing::info!("OAuth endpoint: http://localhost:3000/api/oauth/url");

    let server = async {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app_router).await.unwrap();
    };

    let shutdown_tx = shutdown_tx;
    let app_for_shutdown = app_for_shutdown;

    tokio::select! {
        _ = server => {
            tracing::info!("Server stopped");
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received");
        }
    }

    tracing::info!("Shutting down...");
    let _ = shutdown_tx.send(());
    token_listener.abort();
    if let Some(em) = app_for_shutdown.eventsub.as_ref() {
        tracing::info!("Stopping EventSub...");
        em.stop().await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use utoipa::OpenApi;

    #[test]
    fn generate_openapi() {
        let path = Path::new("generated/openapi.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let json = super::router::ApiDoc::openapi().to_pretty_json().unwrap();
        fs::write(path, json).unwrap();
    }
}
