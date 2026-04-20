use axum::{Json, Router, extract::Extension, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::db::Db;

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"name": "Star"}))]
pub struct KingResponse {
    pub name: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct KingRequest {
    pub name: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!(["Star", "Echo"]))]
pub struct DonatersResponse {
    pub donaters: Vec<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DonaterRequest {
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/king",
    tag = "King",
    responses(
        (status = 200, body = KingResponse, description = "Current king")
    )
)]
pub async fn get_king(Extension(state): Extension<AppState>) -> axum::Json<KingResponse> {
    let king = state.king.read().await;
    axum::Json(KingResponse { name: king.clone() })
}

#[utoipa::path(
    post,
    path = "/api/king",
    tag = "King",
    request_body = KingRequest,
    responses(
        (status = 200, body = KingResponse, description = "Update king")
    )
)]
pub async fn post_king<T: Db>(
    Extension(state): Extension<AppState>,
    State(db): State<T>,
    Json(req): Json<KingRequest>,
) -> axum::Json<KingResponse>
where
    T: Db + Send + Sync,
{
    db.insert_king(&req.name).await.unwrap();
    let mut king = state.king.write().await;
    *king = req.name.clone();
    axum::Json(KingResponse { name: req.name })
}

#[utoipa::path(
    get,
    path = "/api/month",
    tag = "Donaters",
    responses(
        (status = 200, body = DonatersResponse, description = "Donaters for current month")
    )
)]
pub async fn get_month(Extension(state): Extension<AppState>) -> axum::Json<DonatersResponse> {
    let month = state.month.read().await;
    axum::Json(DonatersResponse {
        donaters: month.clone(),
    })
}

#[utoipa::path(
    post,
    path = "/api/month",
    tag = "Donaters",
    request_body = DonaterRequest,
    responses(
        (status = 200, body = DonatersResponse, description = "Add month donater")
    )
)]
pub async fn post_month<T: Db>(
    Extension(state): Extension<AppState>,
    State(db): State<T>,
    Json(req): Json<DonaterRequest>,
) -> axum::Json<DonatersResponse>
where
    T: Db + Send + Sync,
{
    db.insert_month_donater(&req.name).await.unwrap();
    let mut month = state.month.write().await;
    month.push(req.name.clone());
    axum::Json(DonatersResponse {
        donaters: month.clone(),
    })
}

#[utoipa::path(
    post,
    path = "/api/last-day",
    tag = "Donaters",
    request_body = DonaterRequest,
    responses(
        (status = 200, body = DonatersResponse, description = "Add last day donater")
    )
)]
pub async fn post_last_day<T: Db>(
    Extension(state): Extension<AppState>,
    State(db): State<T>,
    Json(req): Json<DonaterRequest>,
) -> axum::Json<DonatersResponse>
where
    T: Db + Send + Sync,
{
    db.insert_last_day_donater(&req.name).await.unwrap();
    let mut last_day = state.last_day.write().await;
    last_day.push(req.name.clone());
    if last_day.len() > 10 {
        last_day.remove(0);
    }
    axum::Json(DonatersResponse {
        donaters: last_day.clone(),
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
pub async fn get_last_day(Extension(state): Extension<AppState>) -> axum::Json<DonatersResponse> {
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
    paths(get_king, post_king, get_month, post_month, get_last_day, post_last_day),
    components(schemas(KingResponse, DonatersResponse, KingRequest, DonaterRequest))
)]
#[allow(dead_code)]
pub struct ApiDoc;

pub fn router<T>(state: AppState, db: T) -> Router
where
    T: Db + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/king", get(get_king).post(post_king::<T>))
        .route("/api/month", get(get_month).post(post_month::<T>))
        .route("/api/last-day", get(get_last_day).post(post_last_day::<T>))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(axum::Extension(state))
        .with_state(db)
}
