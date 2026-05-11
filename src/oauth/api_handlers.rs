use axum::extract::{Extension, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::App;
use crate::error::{AppError, AppResult};

#[derive(Serialize, utoipa::ToSchema)]
pub struct OAuthUrlResponse {
    pub url: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct OAuthCallbackResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct OAuthCallbackParams {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/oauth/url",
    tag = "OAuth",
    responses(
        (status = 200, body = OAuthUrlResponse, description = "Twitch OAuth authorization URL")
    )
)]
pub async fn get_oauth_url(
    Extension(app): Extension<Arc<App>>,
) -> AppResult<axum::Json<OAuthUrlResponse>> {
    let url = app.oauth.get_auth_url().await?;
    Ok(axum::Json(OAuthUrlResponse { url }))
}

#[utoipa::path(
    get,
    path = "/api/oauth/callback",
    tag = "OAuth",
    responses(
        (status = 200, body = OAuthCallbackResponse, description = "OAuth callback result")
    )
)]
pub async fn oauth_callback(
    Extension(app): Extension<Arc<App>>,
    Query(params): Query<OAuthCallbackParams>,
) -> AppResult<axum::Json<OAuthCallbackResponse>> {
    match app.oauth.handle_callback(&params.code).await {
        Ok(true) => Ok(axum::Json(OAuthCallbackResponse {
            success: true,
            message: "Authorization successful! You can now use the EventSub functionality."
                .to_string(),
        })),
        Ok(false) => Ok(axum::Json(OAuthCallbackResponse {
            success: false,
            message: "Authorization failed".to_string(),
        })),
        Err(AppError::Unauthorized(msg)) => {
            tracing::warn!("OAuth callback ignored: {}", msg);
            Ok(axum::Json(OAuthCallbackResponse {
                success: false,
                message: "Authorization failed".to_string(),
            }))
        }
        Err(e) => Err(e),
    }
}
