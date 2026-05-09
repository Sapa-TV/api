mod error;
mod push;
mod supporters;
mod token_manager;

mod db;

// Screaming architecture modules
mod app;
mod auth;
mod eventsub;
mod health;
mod oauth;
mod router;
mod state;

use crate::state::infra::in_memory::InMemoryStateRepository;
use app::app::App;
use app::service_adapters::{CachedSupportersService, SqlitePushService};
use dotenvy::dotenv;
use error::AppResult;
use router::router;
use rustls::crypto::CryptoProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::token_manager::infra::sqlite::SqliteTokenRepository;

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

    let db = crate::db::create_db_pool().await?;

    let supporters_repo = Arc::new(crate::supporters::infra::SqliteSupporterRepository::new(
        db.clone(),
    ));
    let push_repo = Arc::new(crate::push::infra::SqlitePushSubscriptionRepository::new(
        db.clone(),
    ));
    let token_repo = Arc::new(SqliteTokenRepository::new(db.clone()));
    let token_manager = Arc::new(crate::token_manager::application::TokenManager::new(
        token_repo,
    ));

    let state_cache = Arc::new(InMemoryStateRepository::new());
    let cached_supporters = Arc::new(CachedSupportersService::new(
        state_cache,
        supporters_repo.clone(),
    ));

    let push_client = match crate::push::infra::PushClient::from_env() {
        Some(c) => Arc::new(c),
        None => {
            tracing::warn!("PushClient not initialized - VAPID keys not configured");
            Arc::new(
                crate::push::infra::PushClient::new("placeholder", "mailto:placeholder")
                    .expect("hardcoded"),
            )
        }
    };

    let twitch_api_client = Arc::new(crate::eventsub::infra::client::TwitchApiClient::new(
        Arc::new(twitch_api::HelixClient::new()),
        token_manager.clone(),
    ));

    let app: App = App::builder()
        .supporters(cached_supporters)
        .push(Arc::new(SqlitePushService::new(push_repo)))
        .oauth(twitch_api_client)
        .token_manager(token_manager)
        .push_client(push_client)
        .build();

    let app_router = router(Arc::new(app));

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
