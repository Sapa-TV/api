use axum::{
    Json, Router,
    extract::Extension,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::twitch::auth::UserTokenManager;

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
pub struct HealthResponse {
    pub status: String,
    pub version: String,
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
    path = "/api/health",
    tag = "Health",
    responses(
        (status = 200, body = HealthResponse, description = "Health check")
    )
)]
pub async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
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
    path = "/api/push/test-all",
    tag = "Push",
    request_body = PushTestRequest,
    responses(
        (status = 200, body = PushTestResponse, description = "Send test push to all subscriptions")
    )
)]
pub async fn test_push_all<T: Db>(
    State(db): State<T>,
    Json(req): Json<PushTestRequest>,
) -> AppResult<axum::Json<PushTestResponse>>
where
    T: Db + Send + Sync,
{
    use crate::push::PushClient;

    let subscriptions = db.get_all_subscriptions().await?;
    let client = match PushClient::from_env() {
        Some(c) => c,
        None => return Ok(axum::Json(PushTestResponse { sent: 0 })),
    };

    let sent = client
        .send_to_all(&subscriptions, &req.title, &req.body)
        .await;

    Ok(axum::Json(PushTestResponse { sent }))
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
) -> AppResult<axum::Json<KingResponse>>
where
    T: Db + Send + Sync,
{
    db.insert_king(&req.name).await?;
    let mut king = state.king.write().await;
    *king = req.name.clone();
    Ok(axum::Json(KingResponse { name: req.name }))
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
) -> AppResult<axum::Json<DonatersResponse>>
where
    T: Db + Send + Sync,
{
    db.insert_month_donater(&req.name).await?;
    let mut month = state.month.write().await;
    month.push(req.name.clone());
    Ok(axum::Json(DonatersResponse {
        donaters: month.clone(),
    }))
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
) -> AppResult<axum::Json<DonatersResponse>>
where
    T: Db + Send + Sync,
{
    db.insert_last_day_donater(&req.name).await?;
    let mut last_day = state.last_day.write().await;
    last_day.push(req.name.clone());
    if last_day.len() > 10 {
        last_day.remove(0);
    }
    Ok(axum::Json(DonatersResponse {
        donaters: last_day.clone(),
    }))
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

#[utoipa::path(
    post,
    path = "/api/push/subscription",
    tag = "Push",
    request_body = PushSubscriptionRequest,
    responses(
        (status = 200, body = PushSubscriptionResponse, description = "Save push subscription")
    )
)]
pub async fn post_subscription<T: Db>(
    State(db): State<T>,
    Json(req): Json<PushSubscriptionRequest>,
) -> AppResult<axum::Json<PushSubscriptionResponse>>
where
    T: Db + Send + Sync,
{
    db.insert_subscription(
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
pub async fn delete_subscription<T: Db>(
    State(db): State<T>,
    Json(req): Json<PushSubscriptionRequest>,
) -> AppResult<axum::Json<PushSubscriptionResponse>>
where
    T: Db + Send + Sync,
{
    db.delete_subscription(&req.endpoint).await?;
    Ok(axum::Json(PushSubscriptionResponse { success: true }))
}

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
    Extension(token_manager): Extension<std::sync::Arc<UserTokenManager>>,
) -> AppResult<axum::Json<OAuthUrlResponse>> {
    let url = token_manager.get_oauth_url().await?;
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
pub async fn oauth_callback<T: Db + Send + Sync>(
    Extension(token_manager): Extension<std::sync::Arc<UserTokenManager>>,
    State(db): State<T>,
    axum::extract::Query(params): axum::extract::Query<OAuthCallbackParams>,
) -> AppResult<axum::Json<OAuthCallbackResponse>> {
    let code = params.code.as_str();
    token_manager.exchange_code(&db, code).await?;

    Ok(axum::Json(OAuthCallbackResponse {
        success: true,
        message: "Authorization successful! You can now use the EventSub functionality."
            .to_string(),
    }))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct OAuthCallbackParams {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Health", description = "Health check"),
        (name = "King", description = "King operations"),
        (name = "Donaters", description = "Donaters operations"),
        (name = "Push", description = "Web Push notifications"),
        (name = "OAuth", description = "Twitch OAuth authorization")
    ),
    paths(health, get_king, post_king, get_month, post_month, get_last_day, post_last_day, post_subscription, delete_subscription, get_vapid_public_key, test_push_all, get_oauth_url, oauth_callback),
    components(schemas(HealthResponse, KingResponse, DonatersResponse, KingRequest, DonaterRequest, PushSubscriptionRequest, PushSubscriptionResponse, PushKeys, VapidPublicKeyResponse, PushTestRequest, PushTestResponse, OAuthUrlResponse, OAuthCallbackResponse, OAuthCallbackParams)),
    info(
        title = "api",
        version = "v0",
        description = "API for sapa-tv.ru"
    )
)]
#[allow(dead_code)]
pub struct ApiDoc;

pub fn router<T>(state: AppState, db: T, token_manager: std::sync::Arc<UserTokenManager>) -> Router
where
    T: Db + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/health", get(health))
        .route("/api/king", get(get_king).post(post_king::<T>))
        .route("/api/month", get(get_month).post(post_month::<T>))
        .route("/api/last-day", get(get_last_day).post(post_last_day::<T>))
        .route(
            "/api/push/subscription",
            post(post_subscription::<T>).delete(delete_subscription::<T>),
        )
        .route("/api/push/vapid-public-key", get(get_vapid_public_key))
        .route("/api/push/test-all", post(test_push_all::<T>))
        .route("/api/oauth/url", get(get_oauth_url))
        .route("/api/oauth/callback", get(oauth_callback::<T>))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(axum::Extension(state))
        .layer(axum::Extension(token_manager))
        .with_state(db)
}
