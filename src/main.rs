mod api;
mod app_logic;
mod app_services;
mod app_state;
mod auth_service;
mod error;
mod infrastructure;
mod providers;
mod push;
mod supporters;
mod token_manager;

// New screaming architecture modules (v2 - parallel structure)
mod shared_infra_v2;
mod app_v2;
mod eventsub_v2;
mod oauth_v2;
mod auth_v2;
mod state_v2;
mod supporters_v2;
mod push_v2;
mod token_manager_v2;

use api::router;
use app_services::AppServices;
use app_state::create_state;
use dotenvy::dotenv;
use error::AppResult;
use rustls::crypto::CryptoProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::infrastructure::{InitDbData, create_db};

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

    let state = create_state(db.as_ref()).await?;
    let services = AppServices::builder().db(db.clone()).build().await?;

    let app = router(state, services.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Backend API: http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/docs");
    tracing::info!("ReDoc: http://localhost:3000/redoc");
    tracing::info!("OpenAPI JSON: http://localhost:3000/openapi.json");
    tracing::info!("OAuth endpoint: http://localhost:3000/api/oauth/url");

    let server = async {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
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
        let json = super::api::ApiDoc::openapi().to_pretty_json().unwrap();
        fs::write(path, json).unwrap();
    }
}
