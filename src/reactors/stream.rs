use tokio::sync::broadcast;

use crate::event_bus::Event;

pub struct StreamReactor;

impl StreamReactor {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, mut rx: broadcast::Receiver<Event>) {
        loop {
            match rx.recv().await {
                Ok(Event::Stream(evt)) => {
                    tracing::info!(
                        "[{:?}] Stream {:?} at {}",
                        evt.provider,
                        evt.status,
                        evt.timestamp
                    );
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("StreamReactor lagged {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("StreamReactor channel closed");
                    break;
                }
            }
        }
    }
}

impl Default for StreamReactor {
    fn default() -> Self {
        Self::new()
    }
}
