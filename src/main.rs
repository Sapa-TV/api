mod api;
mod app_services;
mod app_state;
mod db;
mod error;
mod push;
mod twitch;

use rustls::crypto::CryptoProvider;

use std::net::SocketAddr;
use std::sync::Arc;

use api::router;
use app_services::AppServices;
use app_state::create_state;
use db::{create_db, init_db};
use error::AppResult;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> AppResult<()> {
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
    init_db(db.as_ref()).await?;

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
