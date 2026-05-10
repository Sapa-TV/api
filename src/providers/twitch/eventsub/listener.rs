use chrono::Utc;
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use twitch_api::eventsub::channel::chat::message::ChannelChatMessageV1;
use twitch_api::eventsub::stream::offline::StreamOfflineV1;
use twitch_api::eventsub::stream::online::StreamOnlineV1;
use twitch_api::eventsub::{Event, EventsubWebsocketData};

use crate::error::{AppError, AppResult};
use crate::event_bus::{
    ChatMessage, ControlEvent, Event as BusEvent, EventBus, StreamEvent, StreamStatus,
};
use crate::providers::twitch::api::TwitchApiClient;
use crate::token::types::ProviderVariant;

const TWITCH_EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

pub struct TwitchEventSubClient {
    api_client: Arc<TwitchApiClient>,
    event_bus: Arc<EventBus>,
}

impl TwitchEventSubClient {
    pub fn new(api_client: Arc<TwitchApiClient>, event_bus: Arc<EventBus>) -> Self {
        Self {
            api_client,
            event_bus,
        }
    }

    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> AppResult<()> {
        let url = TWITCH_EVENTSUB_WS_URL.to_string();

        loop {
            tracing::info!("Connecting to Twitch EventSub WebSocket: {}", url);

            match self.connect_and_handle(&url).await {
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

    async fn connect_and_handle(&self, url: &str) -> AppResult<()> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to EventSub: {}", e)))?;

        let (_, mut read) = ws_stream.split();
        let mut session_id: Option<String> = None;
        let mut last_message_time: Option<std::time::Instant> = None;
        let mut keepalive_timeout = std::time::Duration::from_secs(30);

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(msg)) => match msg {
                            Message::Text(text) => {
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

                                self.handle_message_impl(parsed, &mut session_id, &mut last_message_time).await?;
                            }
                            Message::Ping(_) => {
                            }
                            Message::Close(close_frame) => {
                                tracing::info!("WebSocket closed: {:?}", close_frame);
                                break;
                            }
                            _ => {
                                tracing::warn!("Unexpected WebSocket message: {:?}", msg);
                            }
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {:?}", e);
                            return Err(AppError::Internal(format!("WebSocket error: {}", e)));
                        }
                        None => {
                            tracing::info!("WebSocket stream ended");
                            break;
                        }
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

    async fn handle_message_impl(
        &self,
        parsed: EventsubWebsocketData<'_>,
        session_id: &mut Option<String>,
        last_message_time: &mut Option<std::time::Instant>,
    ) -> AppResult<()> {
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
                    self.event_bus.publish(BusEvent::Control(ControlEvent {
                        provider: ProviderVariant::Twitch,
                        reconnect_url: Some(url.to_string()),
                        disconnect_code: None,
                    }));
                }
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
        tracing::info!("EventSub notification type: {}", sub_type);

        tracing::debug!(
            "Event data: {}",
            serde_json::to_string_pretty(&event).unwrap_or_default()
        );

        match event {
            Event::StreamOnlineV1(_payload) => {
                self.event_bus.publish(BusEvent::Stream(StreamEvent {
                    provider: ProviderVariant::Twitch,
                    status: StreamStatus::Started,
                    timestamp: Utc::now(),
                }));
            }
            Event::StreamOfflineV1(_payload) => {
                self.event_bus.publish(BusEvent::Stream(StreamEvent {
                    provider: ProviderVariant::Twitch,
                    status: StreamStatus::Ended,
                    timestamp: Utc::now(),
                }));
            }
            Event::ChannelChatMessageV1(payload) => {
                if let twitch_api::eventsub::Message::Notification(event) = &payload.message {
                    self.event_bus.publish(BusEvent::Chat(ChatMessage {
                        provider: ProviderVariant::Twitch,
                        user_id: event.chatter_user_id.to_string(),
                        username: event.chatter_user_name.to_string(),
                        message: event.message.text.to_string(),
                        timestamp: Utc::now(),
                    }));
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
    ) -> AppResult<()> {
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
    ) -> AppResult<()> {
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
    ) -> AppResult<()> {
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
