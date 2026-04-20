mod api;
mod app_state;
mod db;

use std::net::SocketAddr;

use api::router;
use app_state::create_state;
use db::{create_db, init_db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = create_db().await?;
    init_db(&db).await?;

    let state = create_state(&db).await?;

    let app = router(state, db);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("===========================================");
    println!("Backend API: http://localhost:3000");
    println!("Swagger UI: http://localhost:3000/docs");
    println!("ReDoc: http://localhost:3000/redoc");
    println!("OpenAPI JSON: http://localhost:3000/openapi.json");
    println!("===========================================");

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
        let json = super::api::ApiDoc::openapi().to_json().unwrap();
        fs::write(path, json).unwrap();
    }
}
