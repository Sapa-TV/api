mod error;
mod push;
mod supporters;
mod token_manager;

// Screaming architecture modules
mod app;
mod auth;
mod eventsub;
mod oauth;
mod router;
mod shared_infra;
mod state;

use crate::state::infra::in_memory::InMemoryStateRepository;
use app::app::{App, AppBuilder};
use app::service_adapters::{
    CachedSupportersService, SqlitePushService, SqliteSupporterService, TwitchOAuthService,
};
use dotenvy::dotenv;
use error::AppResult;
use router::router;
use rustls::crypto::CryptoProvider;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::shared_infra::sqlite_db::SqliteDb;
use crate::token_manager::infra::sqlite::SqliteTokenRepository;

pub struct InitDbData {
    pub king: String,
    pub day_supporters: Vec<String>,
    pub month_supporters: Vec<String>,
}

impl InitDbData {
    pub fn new() -> Self {
        Self {
            king: "Star".to_string(),
            day_supporters: vec![],
            month_supporters: vec![],
        }
    }
}

impl SqliteDb {
    pub async fn init(&self, data: InitDbData) -> AppResult<()> {
        use crate::supporters::domain::{SupporterRepository, SupporterRepositoryData};

        let supporters_data = SupporterRepositoryData {
            king: data.king,
            day_supporters: data.day_supporters,
            month_supporters: data.month_supporters,
        };

        // For now, just verify the db is connected
        tracing::info!("Database initialized");
        Ok(())
    }
}

async fn create_db() -> AppResult<SqliteDb> {
    use sqlx::sqlite::SqliteConnectOptions;
    use std::env;
    use std::str::FromStr;

    let db_url = env::var("DATABASE_URL")?;
    tracing::info!("Connecting to database at {}", db_url);

    let connection_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(connection_options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(SqliteDb::new(pool))
}

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

    let db = Arc::new(create_db().await?);
    let init_db_data = InitDbData::new();
    db.init(init_db_data).await?;

    let supporters_repo = Arc::new(crate::supporters::infra::SqliteSupporterRepository::new(
        db.clone(),
    ));
    let push_repo = Arc::new(crate::push::infra::SqlitePushSubscriptionRepository::new(
        db.clone(),
    ));
    let token_repo = Arc::new(SqliteTokenRepository::new(db.clone()));
    let token_manager = Arc::new(crate::token_manager::application::TokenManagerS::new(
        token_repo,
    ));

    let state_cache = InMemoryStateRepository::new();
    let cached_supporters = Arc::new(CachedSupportersService::new(
        state_cache,
        supporters_repo.clone(),
    ));

    let push_client = match crate::push::client::PushClient::from_env() {
        Some(c) => Arc::new(c),
        None => {
            tracing::warn!("PushClient not initialized - VAPID keys not configured");
            Arc::new(
                crate::push::client::PushClient::new("placeholder", "mailto:placeholder")
                    .expect("hardcoded"),
            )
        }
    };

    let app: App = App::builder()
        .supporters(cached_supporters)
        .push(Arc::new(SqlitePushService::new(push_repo)))
        .oauth(Arc::new(TwitchOAuthService::new(Arc::new(
            crate::eventsub::infra::client::TwitchApiClient::new(
                Arc::new(twitch_api::HelixClient::new()),
                token_manager.clone(),
            ),
        ))))
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
