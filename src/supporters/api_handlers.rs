use axum::{Json, extract::Extension};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::app::App;
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
    Extension(app): Extension<Arc<App>>,
) -> AppResult<Json<SupporterResponse>> {
    let name = app.supporters.get_king().await?.unwrap_or_default();
    Ok(Json(SupporterResponse { name }))
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
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<Json<SupporterResponse>> {
    app.supporters.set_king(&req.name).await?;
    Ok(Json(SupporterResponse { name: req.name }))
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
    Extension(app): Extension<Arc<App>>,
) -> AppResult<Json<SupportersResponse>> {
    let supporters = app.supporters.get_month_supporters().await?;
    Ok(Json(SupportersResponse { supporters }))
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
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<Json<SupportersResponse>> {
    app.supporters.add_month_supporter(&req.name).await?;
    let supporters = app.supporters.get_month_supporters().await?;
    Ok(Json(SupportersResponse { supporters }))
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
    Extension(app): Extension<Arc<App>>,
) -> AppResult<Json<SupportersResponse>> {
    let supporters = app.supporters.get_day_supporters().await?;
    Ok(Json(SupportersResponse { supporters }))
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
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<SupporterRequest>,
) -> AppResult<Json<SupportersResponse>> {
    app.supporters.add_day_supporter(&req.name).await?;
    let supporters = app.supporters.get_day_supporters().await?;
    Ok(Json(SupportersResponse { supporters }))
}
