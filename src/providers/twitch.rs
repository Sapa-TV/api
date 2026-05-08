pub mod auth;
pub mod client;
pub mod eventsub;
pub mod lifecycle;
pub mod token_provider;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use self::lifecycle::TwitchLifecycle;
use crate::app_logic::{ChatHandler, StreamLifecycle};
use crate::error::AppResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppTwitchToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: DateTime<Utc>,
}

pub struct TwitchProvider {
    pub lifecycle: Arc<TwitchLifecycle>,
}

impl TwitchProvider {
    pub async fn new(
        stream_lifecycle: Arc<dyn StreamLifecycle>,
        chat_handler: Arc<dyn ChatHandler>,
    ) -> AppResult<Self> {
        let lifecycle = Arc::new(TwitchLifecycle::new(stream_lifecycle, chat_handler));
        Ok(Self { lifecycle })
    }
}
