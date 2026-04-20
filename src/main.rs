mod api;
mod app_state;

use std::net::SocketAddr;

use api::router;
use app_state::{create_state, state_updater};

#[tokio::main]
async fn main() {
    let state = create_state().await;

    let state_clone = state.clone();
    tokio::spawn(async move {
        state_updater(state_clone).await;
    });

    let app = router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("===========================================");
    println!("Backend API: http://localhost:3000");
    println!("Swagger UI: http://localhost:3000/docs");
    println!("ReDoc: http://localhost:3000/redoc");
    println!("OpenAPI JSON: http://localhost:3000/openapi.json");
    println!("===========================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
