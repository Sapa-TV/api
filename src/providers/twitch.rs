pub mod auth;
pub mod client;
pub mod eventsub;
pub mod lifecycle;

use std::sync::Arc;

use crate::app_logic::{ChatHandler, StreamLifecycle};
use crate::error::AppResult;

use self::lifecycle::TwitchLifecycle;

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
