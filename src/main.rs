mod api;
mod app_state;
mod db;
mod push;

use std::net::SocketAddr;

use api::router;
use app_state::create_state;
use db::{create_db, init_db};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting application");

    let db = create_db().await?;
    init_db(&db).await?;

    let state = create_state(&db).await?;

    let app = router(state, db);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Backend API: http://localhost:3000");
    tracing::info!("Swagger UI: http://localhost:3000/docs");
    tracing::info!("ReDoc: http://localhost:3000/redoc");
    tracing::info!("OpenAPI JSON: http://localhost:3000/openapi.json");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

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
