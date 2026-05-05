pub mod token_repository;
pub mod twitch;

use std::sync::Arc;

use crate::app_logic::{ChatHandler, StreamLifecycle};
use crate::error::AppResult;

pub struct Providers {
    pub twitch: Arc<twitch::TwitchProvider>,
}

impl Providers {
    pub async fn new(
        stream_lifecycle: Arc<dyn StreamLifecycle>,
        chat_handler: Arc<dyn ChatHandler>,
    ) -> AppResult<Self> {
        let twitch = Arc::new(twitch::TwitchProvider::new(stream_lifecycle, chat_handler).await?);

        Ok(Self { twitch })
    }
}
