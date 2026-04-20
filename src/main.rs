use axum::{Router, extract::State, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

const PORT: u16 = 3000;

#[derive(Clone)]
struct AppState {
    king: Arc<RwLock<&'static str>>,
    last_day: Arc<RwLock<Vec<&'static str>>>,
}

const KING_OPTIONS: [&str; 4] = ["Star", "Silent Echo", "Midnight Phantom", "Dragon Lord"];

const LAST_DAY_OPTIONS: [&str; 8] = [
    "Star",
    "Echo",
    "Vortex",
    "Night Wolf",
    "Shadow Hunter",
    "Eternal Winter",
    "Crimson Sunrise",
    "Dragon Lord",
];

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"name": "Star"}))]
struct KingResponse {
    name: &'static str,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!(["Star", "Echo"]))]
struct DonatersResponse {
    donaters: Vec<&'static str>,
}

#[utoipa::path(
    get,
    path = "/api/king",
    tag = "King",
    responses(
        (status = 200, body = KingResponse, description = "Current king")
    )
)]
async fn get_king(State(state): State<AppState>) -> axum::Json<KingResponse> {
    let king = state.king.read().await;
    axum::Json(KingResponse { name: *king })
}

#[utoipa::path(
    get,
    path = "/api/month",
    tag = "Donaters",
    responses(
        (status = 200, body = DonatersResponse, description = "Donaters for current month")
    )
)]
async fn get_month() -> axum::Json<DonatersResponse> {
    axum::Json(DonatersResponse {
        donaters: vec!["Star", "Echo", "Vortex", "Night Wolf", "Shadow Hunter"],
    })
}

#[utoipa::path(
    get,
    path = "/api/last-day",
    tag = "Donaters",
    responses(
        (status = 200, body = DonatersResponse, description = "Donaters for last day")
    )
)]
async fn get_last_day(State(state): State<AppState>) -> axum::Json<DonatersResponse> {
    let last_day = state.last_day.read().await;
    axum::Json(DonatersResponse {
        donaters: last_day.clone(),
    })
}

async fn state_updater(state: AppState) {
    let mut king_idx = 0usize;
    let mut last_day_idx = 0usize;
    let mut timer = interval(Duration::from_secs(20));

    loop {
        timer.tick().await;

        {
            let mut king = state.king.write().await;
            *king = KING_OPTIONS[king_idx % KING_OPTIONS.len()];
            king_idx += 1;
        }

        {
            let mut last_day = state.last_day.write().await;
            let new_nick = LAST_DAY_OPTIONS[last_day_idx % LAST_DAY_OPTIONS.len()];
            last_day.push(new_nick);
            if last_day.len() > 10 {
                last_day.remove(0);
            }
            last_day_idx += 1;
        }

        println!("State updated: king and last_day");
    }
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "King", description = "King operations"),
        (name = "Donaters", description = "Donaters operations")
    ),
    paths(get_king, get_month, get_last_day),
    components(schemas(KingResponse, DonatersResponse))
)]
#[allow(dead_code)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let state = AppState {
        king: Arc::new(RwLock::new(KING_OPTIONS[0])),
        last_day: Arc::new(RwLock::new(vec!["Echo", "Night Wolf", "Shadow Hunter"])),
    };

    let state_clone = state.clone();
    tokio::spawn(async move {
        state_updater(state_clone).await;
    });

    let app = Router::new()
        .route("/api/king", get(get_king))
        .route("/api/month", get(get_month))
        .route("/api/last-day", get(get_last_day))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    println!("===========================================");
    println!("Backend API: http://localhost:{}", PORT);
    println!("Swagger UI: http://localhost:{}/docs", PORT);
    println!("ReDoc: http://localhost:{}/redoc", PORT);
    println!("OpenAPI JSON: http://localhost:{}/openapi.json", PORT);
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
        let json = super::ApiDoc::openapi().to_json().unwrap();
        fs::write(path, json).unwrap();
    }
}
