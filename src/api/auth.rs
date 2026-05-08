use axum::extract::Extension;
use serde::{Deserialize, Serialize};

use crate::app_services::AppServices;
use crate::error::{AppError, AppResult};

#[derive(Serialize, utoipa::ToSchema)]
pub struct OAuthUrlResponse {
    pub url: String,
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
    Extension(services): Extension<AppServices>,
) -> AppResult<axum::Json<OAuthUrlResponse>> {
    let url = services.twitch_api.get_oauth_url().await?;
    Ok(axum::Json(OAuthUrlResponse { url }))
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct OAuthCallbackResponse {
    pub success: bool,
    pub message: String,
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
    Extension(services): Extension<AppServices>,
    axum::extract::Query(params): axum::extract::Query<OAuthCallbackParams>,
) -> AppResult<axum::Json<OAuthCallbackResponse>> {
    let code = params.code.as_str();
    match services
        .twitch_api
        .exchange_code(services.db.as_ref(), code)
        .await
    {
        Ok(scopes_valid) => {
            if !scopes_valid {
                tracing::warn!("OAuth succeeded but scopes don't match requirements");
            }
            Ok(axum::Json(OAuthCallbackResponse {
                success: true,
                message: "Authorization successful! You can now use the EventSub functionality."
                    .to_string(),
            }))
        }
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct OAuthCallbackParams {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}
