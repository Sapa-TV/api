use axum::{Router, extract::State, routing::get};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"name": "Star"}))]
pub struct KingResponse {
    pub name: &'static str,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!(["Star", "Echo"]))]
pub struct DonatersResponse {
    pub donaters: Vec<&'static str>,
}

#[utoipa::path(
    get,
    path = "/api/king",
    tag = "King",
    responses(
        (status = 200, body = KingResponse, description = "Current king")
    )
)]
pub async fn get_king(State(state): State<AppState>) -> axum::Json<KingResponse> {
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
pub async fn get_month() -> axum::Json<DonatersResponse> {
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
pub async fn get_last_day(State(state): State<AppState>) -> axum::Json<DonatersResponse> {
    let last_day = state.last_day.read().await;
    axum::Json(DonatersResponse {
        donaters: last_day.clone(),
    })
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
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/king", get(get_king))
        .route("/api/month", get(get_month))
        .route("/api/last-day", get(get_last_day))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .with_state(state)
}
