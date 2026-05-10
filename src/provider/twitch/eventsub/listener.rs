use chrono::Utc;
use futures_util::{Sink, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use twitch_api::eventsub::channel::chat::message::ChannelChatMessageV1;
use twitch_api::eventsub::stream::offline::StreamOfflineV1;
use twitch_api::eventsub::stream::online::StreamOnlineV1;
use twitch_api::eventsub::{Event, EventsubWebsocketData};

use crate::error::{AppError, AppResult};
use crate::provider::twitch::api::TwitchApiClient;
use crate::provider::twitch::eventsub::manager::TwitchLifecycle;

const TWITCH_EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

pub struct EventSubClient {
    api_client: Arc<TwitchApiClient>,
    lifecycle: Arc<TwitchLifecycle>,
}

impl EventSubClient {
    pub fn new(api_client: Arc<TwitchApiClient>, lifecycle: Arc<TwitchLifecycle>) -> Self {
        Self {
            api_client,
            lifecycle,
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

                if let Some(broadcaster_user_id) = self.api_client.get_broadcaster_id().await {
                    if let Some(session_id) = session_id.as_ref() {
                        self.subscribe_stream_started(session_id, &broadcaster_user_id)
                            .await?;
                        self.subscribe_stream_ended(session_id, &broadcaster_user_id)
                            .await?;
                        self.subscribe_chat_messages(session_id, &broadcaster_user_id)
                            .await?;
                    }
                }
            }
            EventsubWebsocketData::Keepalive {
                metadata: _,
                payload: _,
            } => {
                *last_message_time = Some(std::time::Instant::now());
            }
            EventsubWebsocketData::Notification {
                metadata: _,
                payload,
            } => {
                self.handle_notification(payload).await?;
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
                tracing::warn!("Subscription revoked: type={}", sub_type);
            }
            _ => {
                tracing::warn!("Unknown EventSub message type: {:?}", parsed);
            }
        }

        Ok(())
    }

    async fn handle_notification(&self, event: Event) -> AppResult<()> {
        let sub_type = format!("{:?}", event);
        tracing::info!("EventSub notification: type={}", sub_type);

        tracing::debug!(
            "Event data: {}",
            serde_json::to_string_pretty(&event).unwrap_or_default()
        );

        match event {
            Event::StreamOnlineV1(_payload) => {
                let timestamp = Utc::now();
                self.lifecycle.on_stream_started(timestamp).await?;
            }
            Event::StreamOfflineV1(_payload) => {
                let timestamp = Utc::now();
                self.lifecycle.on_stream_ended(timestamp).await?;
            }
            Event::ChannelChatMessageV1(payload) => {
                if let twitch_api::eventsub::Message::Notification(event) = &payload.message {
                    let user_id = event.chatter_user_id.as_str();
                    let username = event.chatter_user_name.as_str();
                    let message = event.message.text.as_str();
                    let timestamp = Utc::now();
                    self.lifecycle
                        .on_chat_message(user_id, username, message, timestamp)
                        .await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn subscribe_stream_started(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Creating stream.online subscription: broadcaster_id={}",
            broadcaster_user_id
        );

        let subscription = StreamOnlineV1::broadcaster_user_id(broadcaster_user_id.to_string());
        let transport = twitch_api::eventsub::Transport::websocket(session_id);

        let subscription_id = self
            .api_client
            .create_eventsub_subscription(subscription, transport)
            .await?;

        tracing::info!("Subscribed to stream.online: id={}", subscription_id);
        Ok(())
    }

    async fn subscribe_stream_ended(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Creating stream.offline subscription: broadcaster_id={}",
            broadcaster_user_id
        );

        let subscription = StreamOfflineV1::broadcaster_user_id(broadcaster_user_id.to_string());
        let transport = twitch_api::eventsub::Transport::websocket(session_id);

        let subscription_id = self
            .api_client
            .create_eventsub_subscription(subscription, transport)
            .await?;

        tracing::info!("Subscribed to stream.offline: id={}", subscription_id);
        Ok(())
    }

    async fn subscribe_chat_messages(
        &self,
        session_id: &str,
        broadcaster_user_id: &str,
    ) -> Result<(), AppError> {
        tracing::info!(
            "Creating chat.message subscription: broadcaster_id={}",
            broadcaster_user_id,
        );

        let subscription = ChannelChatMessageV1::new(
            broadcaster_user_id.to_string(),
            broadcaster_user_id.to_string(),
        );
        let transport = twitch_api::eventsub::Transport::websocket(session_id);

        let subscription_id = self
            .api_client
            .create_eventsub_subscription(subscription, transport)
            .await?;

        tracing::info!("Subscribed to channel.chat.message: id={}", subscription_id);

        Ok(())
    }
}

pub async fn start_eventsub_task(
    api_client: Arc<TwitchApiClient>,
    lifecycle: Arc<TwitchLifecycle>,
    shutdown: broadcast::Receiver<()>,
) {
    let client = EventSubClient::new(api_client, lifecycle);

    if let Err(e) = client.run(shutdown).await {
        tracing::error!("EventSub task error: {:?}", e);
    }
}
