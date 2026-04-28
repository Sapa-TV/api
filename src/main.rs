mod api;
mod app_state;
mod db;
mod error;
mod push;
mod twitch;

use std::net::SocketAddr;
use std::sync::Arc;

use api::router;
use app_state::create_state;
use db::{create_db, init_db};
use error::AppResult;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use twitch::auth::UserTokenManager;
use twitch::eventsub::{create_eventsub_shutdown_channel, start_eventsub_task};

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting application");

    let db = create_db().await?;
    init_db(&db).await?;

    let state = create_state(&db).await?;

    let (eventsub_shutdown_tx, eventsub_shutdown_rx) = create_eventsub_shutdown_channel();

    let client_id = std::env::var("TWITCH_CLIENT_ID")
        .map_err(|_| error::AppError::Env("TWITCH_CLIENT_ID not set".to_string()))?;
    let client_secret = std::env::var("TWITCH_CLIENT_SECRET")
        .map_err(|_| error::AppError::Env("TWITCH_CLIENT_SECRET not set".to_string()))?;
    let redirect_uri = std::env::var("TWITCH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/api/oauth/callback".to_string());

    let token_manager = Arc::new(UserTokenManager::new(
        client_id,
        client_secret,
        redirect_uri,
    ));
    token_manager.load_from_db(&db).await?;

    let app = router(state, db.clone(), token_manager.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Backend API: http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/docs");
    tracing::info!("ReDoc: http://localhost:3000/redoc");
    tracing::info!("OpenAPI JSON: http://localhost:3000/openapi.json");

    let has_token = token_manager.get_access_token().await.is_some();
    if has_token {
        tracing::info!("Twitch user token found in database, starting EventSub...");
        let tm = token_manager.clone();
        tokio::spawn(async move {
            tracing::info!("Starting Twitch EventSub listener");
            start_eventsub_task(tm, eventsub_shutdown_rx).await;
        });
    } else {
        tracing::warn!("No Twitch user token found. EventSub not started.");
        tracing::warn!(
            "To enable EventSub, complete OAuth authorization and save token to database."
        );
    }

    let server = async {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    };

    tokio::select! {
        _ = server => {
            tracing::info!("Server stopped");
        }
    }

    let _ = eventsub_shutdown_tx.send(());

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
