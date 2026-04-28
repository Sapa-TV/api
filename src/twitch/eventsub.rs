use futures_util::{Sink, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use twitch_api::eventsub::channel::chat::message::ChannelChatMessageV1;
use twitch_api::eventsub::{Event, EventsubWebsocketData};
use twitch_api::HelixClient;

use crate::error::AppError;
use crate::twitch::auth::UserTokenManager;

const TWITCH_EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

pub struct EventSubClient {
    _token_manager: Arc<UserTokenManager>,
    helix: Arc<HelixClient<'static, reqwest::Client>>,
}

impl EventSubClient {
    pub fn new(
        token_manager: Arc<UserTokenManager>,
        helix: Arc<HelixClient<'static, reqwest::Client>>,
    ) -> Self {
        Self {
            _token_manager: token_manager,
            helix,
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
        let mut last_message_time: Option<std::time::Instant> = None;
        let mut keepalive_timeout = std::time::Duration::from_secs(30);

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            last_message_time = Some(std::time::Instant::now());

                            let parsed = match Event::parse_websocket(&text) {
                                Ok(p) => p,
                                Err(e) => {
                                    tracing::error!("Failed to parse EventSub message: {}. Raw: {}", e, text);
                                    continue;
                                }
                            };

                            if let EventsubWebsocketData::Welcome { payload, .. } = &parsed {
                                if let Some(timeout) = payload.session.keepalive_timeout_seconds {
                                    keepalive_timeout = std::time::Duration::from_secs((timeout * 2) as u64);
                                    tracing::info!("Keepalive timeout set to {} seconds (2x multiplier)", timeout * 2);
                                }
                            }

                            self.handle_message_impl(parsed, &mut session_id, &mut last_message_time, &mut write).await?;
                        }
                        Some(Ok(Message::Ping(_))) => {
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
                _ = tokio::time::sleep(keepalive_timeout) => {
                    if let Some(last) = last_message_time {
                        if last.elapsed() > keepalive_timeout {
                            tracing::warn!("Keepalive timeout exceeded, reconnecting...");
                            return Err(AppError::Internal("Keepalive timeout".to_string()));
                        }
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
        last_message_time: &mut Option<std::time::Instant>,
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

                if let Some(broadcaster_user_id) = self._token_manager.get_broadcaster_id().await {
                    if let Some(session_id) = session_id.as_ref() {
                        self.subscribe_chat_messages(session_id, &broadcaster_user_id).await?;
                    }
                }
            }
            EventsubWebsocketData::Keepalive {
                metadata: _,
                payload: _,
            } => {
                tracing::info!("Keepalive received");
                *last_message_time = Some(std::time::Instant::now());
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

    async fn subscribe_chat_messages(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<(), AppError> {
        let token = self._token_manager.get_token().await
            .ok_or_else(|| AppError::Internal("No token available".to_string()))?;

        tracing::info!(
            "Creating subscription: broadcaster_id={}, token_user_id={}",
            broadcaster_user_id,
            token.user_id
        );

        let subscription = ChannelChatMessageV1::new(
            broadcaster_user_id.to_string(),
            broadcaster_user_id.to_string(),
        );

        let transport = twitch_api::eventsub::Transport::websocket(session_id);

        let result = self.helix
            .create_eventsub_subscription(subscription, transport, &*token)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create subscription: {}", e)))?;

        tracing::info!("Subscribed to channel.chat.message: id={}", result.id);

        Ok(())
    }
}

pub async fn start_eventsub_task(
    token_manager: Arc<UserTokenManager>,
    helix: Arc<HelixClient<'static, reqwest::Client>>,
    shutdown: broadcast::Receiver<()>,
) {
    let client = EventSubClient::new(token_manager, helix);

    if let Err(e) = client.run(shutdown).await {
        tracing::error!("EventSub task error: {:?}", e);
    }
}

pub fn create_eventsub_shutdown_channel() -> (broadcast::Sender<()>, broadcast::Receiver<()>) {
    broadcast::channel(1)
}
