use tokio::sync::broadcast;

use crate::event_bus::Event;

pub struct ChatReactor;

impl ChatReactor {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, mut rx: broadcast::Receiver<Event>) {
        loop {
            match rx.recv().await {
                Ok(Event::Chat(msg)) => {
                    tracing::info!(
                        "[{:?}] Chat message from {} ({}): {}",
                        msg.provider,
                        msg.username,
                        msg.user_id,
                        msg.message
                    );
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("ChatReactor lagged {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("ChatReactor channel closed");
                    break;
                }
            }
        }
    }
}

impl Default for ChatReactor {
    fn default() -> Self {
        Self::new()
    }
}
