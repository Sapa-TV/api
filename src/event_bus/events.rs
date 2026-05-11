use chrono::{DateTime, Utc};

use crate::token::types::ProviderVariant;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub provider: ProviderVariant,
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum StreamStatus {
    Started,
    Ended,
}

#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub provider: ProviderVariant,
    pub status: StreamStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ControlEvent {
    pub provider: ProviderVariant,
    pub reconnect_url: Option<String>,
    pub disconnect_code: Option<u16>,
}

#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum Event {
    Chat(ChatMessage),
    Stream(StreamEvent),
    Control(ControlEvent),
}
