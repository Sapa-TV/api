mod app;
mod database;
mod error;
mod event_bus;
mod health;
mod oauth;
mod providers;
mod push;
mod reactors;
mod router;
mod state;
mod supporters;
mod token;

use app::app::App;
use app::service_adapters::{CachedSupportersService, SqlitePushService};
use dotenvy::dotenv;
use error::AppResult;
use router::router;
use rustls::crypto::CryptoProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::event_bus::EventBus;
use crate::providers::twitch::{
    api::TwitchApiClient, eventsub::listener::TwitchEventSubClient,
    token_provider::TwitchTokenProvider,
};
use crate::state::in_memory_repository::InMemoryStateRepository;
use crate::token::{
    manager::TokenManager,
    sqlite_repository::SqliteTokenRepository,
    types::{AccountVariant, ProviderVariant},
};

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
    let event_bus = Arc::new(EventBus::new());

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

    let eventsub_client = Arc::new(TwitchEventSubClient::new(
        twitch_api_client.clone(),
        event_bus.clone(),
    ));

    let has_twitch_main_token = token_manager
        .get_token(ProviderVariant::Twitch, AccountVariant::Main)
        .await
        .is_ok();

    if has_twitch_main_token {
        tracing::info!("Twitch main token found, starting EventSub...");
    } else {
        tracing::info!("No main token found, EventSub will start on token update");
    }

    let app: App = App::builder()
        .supporters(cached_supporters)
        .push(Arc::new(SqlitePushService::new(push_repo)))
        .oauth(twitch_api_client)
        .token_manager(token_manager)
        .push_client(push_client)
        .build();

    let app = Arc::new(app);
    let app_router = router(app.clone());

    crate::reactors::register(&event_bus);

    let (event_sub_shutdown_tx, event_sub_shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
    let eventsub_client_clone = eventsub_client.clone();

    let eventsub_handle = tokio::spawn(async move {
        if let Err(e) = eventsub_client_clone.run(event_sub_shutdown_rx).await {
            tracing::error!("EventSub task error: {:?}", e);
        }
    });

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

    tokio::select! {
        _ = server => {
            tracing::info!("Server stopped");
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received");
        }
    }

    tracing::info!("Shutting down...");
    let _ = event_sub_shutdown_tx.send(());
    eventsub_handle.abort();
    event_bus.shutdown().await;

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
