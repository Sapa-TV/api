use axum::extract::Extension;
use serde::Serialize;

use crate::app_services::AppServices;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub commit: String,
    pub need_token_update: bool,
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, body = HealthResponse, description = "Health check")
    )
)]
pub async fn health(Extension(services): Extension<AppServices>) -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit: env!("GIT_HASH").to_string(),
        need_token_update: services.twitch_api.needs_reauth(),
    })
}
