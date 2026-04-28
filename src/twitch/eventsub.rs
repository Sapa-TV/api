use futures_util::{Sink, SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use twitch_api::eventsub::{Event, EventsubWebsocketData};

use crate::error::AppError;
use crate::twitch::auth::UserTokenManager;

const TWITCH_EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

pub struct EventSubClient {
    _token_manager: Arc<UserTokenManager>,
}

impl EventSubClient {
    pub fn new(token_manager: Arc<UserTokenManager>) -> Self {
        Self {
            _token_manager: token_manager,
        }
    }

    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> Result<(), AppError> {
        loop {
            tracing::info!("Connecting to Twitch EventSub WebSocket...");

            match self.connect_and_handle().await {
                Ok(()) => {
                    tracing::info!("EventSub WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    tracing::error!("EventSub WebSocket error: {:?}", e);
                    tracing::info!("Reconnecting in 5 seconds...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }

            if shutdown.try_recv().is_ok() {
                tracing::info!("Received shutdown signal for EventSub");
                break;
            }
        }
        Ok(())
    }

    async fn connect_and_handle(&self) -> Result<(), AppError> {
        let (ws_stream, _) = connect_async(TWITCH_EVENTSUB_WS_URL)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to EventSub: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();
        let mut session_id: Option<String> = None;
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            let parsed = match Event::parse_websocket(&text) {
                                Ok(p) => p,
                                Err(e) => {
                                    tracing::error!("Failed to parse EventSub message: {}. Raw: {}", e, text);
                                    continue;
                                }
                            };
                            self.handle_message_impl(parsed, &mut session_id, &mut write).await?;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            tracing::debug!("Received ping, sending pong");
                            write.send(Message::Pong(data)).await
                                .map_err(|e| AppError::Internal(format!("Failed to send pong: {:?}", e)))?;
                        }
                        Some(Ok(Message::Close(close_frame))) => {
                            tracing::info!("WebSocket closed: {:?}", close_frame);
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {:?}", e);
                            return Err(AppError::Internal(format!("WebSocket error: {}", e)));
                        }
                        None => {
                            tracing::info!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
                _ = heartbeat_interval.tick() => {
                    if session_id.is_some() {
                        let ping_msg = serde_json::json!({
                            "type": "PING"
                        });
                        let msg_str = serde_json::to_string(&ping_msg)
                            .map_err(|e| AppError::Internal(format!("Failed to serialize ping: {}", e)))?;
                        write.send(Message::Text(msg_str.into())).await
                            .map_err(|e| AppError::Internal(format!("Failed to send ping: {}", e)))?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_message_impl<S>(
        &self,
        parsed: EventsubWebsocketData<'_>,
        session_id: &mut Option<String>,
        _write: &mut S,
    ) -> Result<(), AppError>
    where
        S: Sink<Message> + Unpin,
        S::Error: std::fmt::Debug,
    {
        tracing::debug!("EventSub message type: {:?}", parsed);

        match parsed {
            EventsubWebsocketData::Welcome {
                metadata: _,
                payload,
            } => {
                tracing::info!(
                    "EventSub session connected: id={}, status={}",
                    payload.session.id,
                    payload.session.status
                );
                *session_id = Some(payload.session.id.to_string());
            }
            EventsubWebsocketData::Keepalive {
                metadata: _,
                payload: _,
            } => {
                tracing::debug!("Session keepalive received");
            }
            EventsubWebsocketData::Notification {
                metadata: _,
                payload,
            } => {
                let sub_type = format!("{:?}", payload);
                tracing::info!("📨 EventSub notification: type={}", sub_type);

                tracing::debug!(
                    "Event data: {}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
            }
            EventsubWebsocketData::Reconnect {
                metadata: _,
                payload,
            } => {
                tracing::warn!("Reconnect requested by Twitch");
                if let Some(url) = &payload.session.reconnect_url {
                    tracing::info!("Will reconnect to: {}", url);
                }
                return Err(AppError::Internal("Reconnect requested".to_string()));
            }
            EventsubWebsocketData::Revocation {
                metadata: _,
                payload,
            } => {
                let sub_type = format!("{:?}", payload);
                tracing::warn!("⚠️ Subscription revoked: type={}", sub_type);
            }
            _ => {
                tracing::warn!("Unknown EventSub message type: {:?}", parsed);
            }
        }

        Ok(())
    }
}

pub async fn start_eventsub_task(
    token_manager: Arc<UserTokenManager>,
    shutdown: broadcast::Receiver<()>,
) {
    let client = EventSubClient::new(token_manager);

    if let Err(e) = client.run(shutdown).await {
        tracing::error!("EventSub task error: {:?}", e);
    }
}

pub fn create_eventsub_shutdown_channel() -> (broadcast::Sender<()>, broadcast::Receiver<()>) {
    broadcast::channel(1)
}
