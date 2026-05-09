use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::app::app::App;
use crate::health::{HealthResponse, health};
use crate::oauth::api::*;
use crate::push::api::*;
use crate::supporters::api::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Health", description = "Health check"),
        (name = "Supporters", description = "Supporters operations"),
        (name = "Push", description = "Web Push notifications"),
        (name = "OAuth", description = "Provider OAuth authorization")
    ),
    paths(crate::health::health, get_king_supporter, post_king_supporter, get_month_supporters, post_month_supporter, get_day_supporters, post_day_supporter, post_subscription, delete_subscription, get_vapid_public_key, test_push_all, get_oauth_url, oauth_callback),
    components(schemas(crate::health::HealthResponse, SupporterResponse, SupportersResponse, SupporterRequest, PushSubscriptionRequest, PushSubscriptionResponse, PushKeys, VapidPublicKeyResponse, PushTestRequest, PushTestResponse, OAuthUrlResponse, OAuthCallbackResponse, OAuthCallbackParams)),
    info(
        title = "api",
        version = "v0",
        description = "API for sapa-tv.ru"
    )
)]
#[allow(dead_code)]
pub struct ApiDoc;

pub fn router(app: Arc<App>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/supporters/king",
            get(get_king_supporter).post(post_king_supporter),
        )
        .route(
            "/api/supporters/month",
            get(get_month_supporters).post(post_month_supporter),
        )
        .route(
            "/api/supporters/day",
            get(get_day_supporters).post(post_day_supporter),
        )
        .route(
            "/api/push/subscription",
            post(post_subscription).delete(delete_subscription),
        )
        .route("/api/push/vapid-public-key", get(get_vapid_public_key))
        .route("/api/push/test-all", post(test_push_all))
        .route("/api/oauth/url", get(get_oauth_url))
        .route("/api/oauth/callback", get(oauth_callback))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(axum::Extension(app))
}
