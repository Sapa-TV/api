use axum::{Json, extract::Extension};
use serde::{Deserialize, Serialize};

use crate::app_services::AppServices;
use crate::app_state::AppState;
use crate::error::AppResult;

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"name": "Star"}))]
pub struct SupporterResponse {
    pub name: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!(["Star", "Echo"]))]
pub struct SupportersResponse {
    pub supporters: Vec<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SupporterRequest {
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/supporters/king",
    tag = "Supporters",
    responses(
        (status = 200, body = SupporterResponse, description = "Current king supporter")
    )
)]
pub async fn get_king_supporter(
    Extension(state): Extension<AppState>,
) -> axum::Json<SupporterResponse> {
    let king = state.king.read().await;
    axum::Json(SupporterResponse { name: king.clone() })
}

#[utoipa::path(
    post,
    path = "/api/supporters/king",
    tag = "Supporters",
    request_body = SupporterRequest,
    responses(
        (status = 200, body = SupporterResponse, description = "Update king supporter")
    )
)]
pub async fn post_king_supporter(
    Extension(state): Extension<AppState>,
    Extension(services): Extension<AppServices>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<axum::Json<SupporterResponse>> {
    services.db.insert_king(&req.name).await?;
    let mut king = state.king.write().await;
    *king = req.name.clone();
    Ok(axum::Json(SupporterResponse { name: req.name }))
}

#[utoipa::path(
    get,
    path = "/api/supporters/month",
    tag = "Supporters",
    responses(
        (status = 200, body = SupportersResponse, description = "Supporters for current month")
    )
)]
pub async fn get_month_supporters(
    Extension(state): Extension<AppState>,
) -> axum::Json<SupportersResponse> {
    let month = state.month_supporters.read().await;
    axum::Json(SupportersResponse {
        supporters: month.clone(),
    })
}

#[utoipa::path(
    post,
    path = "/api/supporters/month",
    tag = "Supporters",
    request_body = SupporterRequest,
    responses(
        (status = 200, body = SupportersResponse, description = "Add month supporter")
    )
)]
pub async fn post_month_supporter(
    Extension(state): Extension<AppState>,
    Extension(services): Extension<AppServices>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<axum::Json<SupportersResponse>> {
    services.db.insert_month_supporter(&req.name).await?;
    let mut month = state.month_supporters.write().await;
    month.push(req.name.clone());
    Ok(axum::Json(SupportersResponse {
        supporters: month.clone(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/supporters/day",
    tag = "Supporters",
    request_body = SupporterRequest,
    responses(
        (status = 200, body = SupportersResponse, description = "Add last day supporter")
    )
)]
pub async fn post_day_supporter(
    Extension(state): Extension<AppState>,
    Extension(services): Extension<AppServices>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<axum::Json<SupportersResponse>> {
    services.db.insert_day_supporter(&req.name).await?;
    let mut day_supporter = state.day_supporters.write().await;
    day_supporter.push(req.name.clone());
    if day_supporter.len() > 10 {
        day_supporter.remove(0);
    }
    Ok(axum::Json(SupportersResponse {
        supporters: day_supporter.clone(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/supporters/day",
    tag = "Supporters",
    responses(
        (status = 200, body = SupportersResponse, description = "Supporters for last day")
    )
)]
pub async fn get_day_supporters(
    Extension(state): Extension<AppState>,
) -> axum::Json<SupportersResponse> {
    let day_supporters = state.day_supporters.read().await;
    axum::Json(SupportersResponse {
        supporters: day_supporters.clone(),
    })
}
