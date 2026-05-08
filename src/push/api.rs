use axum::{Json, extract::Extension};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::app::App;
use crate::error::{AppError, AppResult};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PushSubscriptionRequest {
    pub endpoint: String,
    pub keys: PushKeys,
    #[schema(example = json!(null))]
    pub user_id: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PushSubscriptionResponse {
    pub success: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct VapidPublicKeyResponse {
    pub key: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PushTestRequest {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PushTestResponse {
    pub sent: u32,
}

#[utoipa::path(
    get,
    path = "/api/push/vapid-public-key",
    tag = "Push",
    responses(
        (status = 200, body = VapidPublicKeyResponse, description = "VAPID public key for push notifications"),
        (status = 500, description = "VAPID_PUBLIC_KEY not configured")
    )
)]
pub async fn get_vapid_public_key() -> AppResult<axum::Json<VapidPublicKeyResponse>> {
    let key = std::env::var("VAPID_PUBLIC_KEY")
        .map_err(|_| AppError::Env("VAPID_PUBLIC_KEY not configured".to_string()))?;
    Ok(axum::Json(VapidPublicKeyResponse { key }))
}

#[utoipa::path(
    post,
    path = "/api/push/subscription",
    tag = "Push",
    request_body = PushSubscriptionRequest,
    responses(
        (status = 200, body = PushSubscriptionResponse, description = "Save push subscription")
    )
)]
pub async fn post_subscription(
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<PushSubscriptionRequest>,
) -> AppResult<axum::Json<PushSubscriptionResponse>> {
    app.push
        .subscribe(
            &req.endpoint,
            &req.keys.p256dh,
            &req.keys.auth,
            req.user_id.as_deref(),
        )
        .await?;
    Ok(axum::Json(PushSubscriptionResponse { success: true }))
}

#[utoipa::path(
    delete,
    path = "/api/push/subscription",
    tag = "Push",
    request_body = PushSubscriptionRequest,
    responses(
        (status = 200, body = PushSubscriptionResponse, description = "Delete push subscription")
    )
)]
pub async fn delete_subscription(
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<PushSubscriptionRequest>,
) -> AppResult<axum::Json<PushSubscriptionResponse>> {
    app.push.unsubscribe(&req.endpoint).await?;
    Ok(axum::Json(PushSubscriptionResponse { success: true }))
}

#[utoipa::path(
    post,
    path = "/api/push/test-all",
    tag = "Push",
    request_body = PushTestRequest,
    responses(
        (status = 200, body = PushTestResponse, description = "Send test push to all subscriptions")
    )
)]
pub async fn test_push_all(
    Extension(app): Extension<Arc<App>>,
    Json(req): Json<PushTestRequest>,
) -> AppResult<axum::Json<PushTestResponse>> {
    let subscriptions = app.push.get_all_subscriptions().await?;

    let sent = app
        .push_client
        .send_to_all(&subscriptions, &req.title, &req.body)
        .await;

    Ok(axum::Json(PushTestResponse { sent }))
}
