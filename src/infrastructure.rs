use crate::{
    auth_service::{AdminSessionRepository, AdminWhiteListRepository},
    providers::token_repository::TokenRepository,
    push::PushSubscriptionRepository,
    supporters::SupporterRepository,
};

mod db;
mod db_sqlite;

pub use db::*;

pub trait FullRepository:
    SupporterRepository
    + PushSubscriptionRepository
    + AdminSessionRepository
    + AdminWhiteListRepository
    + TokenRepository
    + Send
    + Sync
{
}

impl<
    T: SupporterRepository
        + PushSubscriptionRepository
        + AdminSessionRepository
        + AdminWhiteListRepository
        + TokenRepository
        + Send
        + Sync,
> FullRepository for T
{
}

pub struct InitDbData {
    pub king: String,
    pub day_supporters: Vec<String>,
    pub month_supporters: Vec<String>,
}

pub const KING_DEFAULT: &str = "Star";
pub const DAY_LIST_DEFAULT: &[&str] = &["Echo", "Night Wolf", "Shadow Hunter"];
pub const MONTH_LIST_DEFAULT: &[&str] = &["Star", "Echo", "Vortex", "Night Wolf", "Shadow Hunter"];

impl InitDbData {
    pub fn new() -> Self {
        Self {
            king: KING_DEFAULT.to_string(),
            day_supporters: DAY_LIST_DEFAULT.iter().map(|s| s.to_string()).collect(),
            month_supporters: MONTH_LIST_DEFAULT.iter().map(|s| s.to_string()).collect(),
        }
    }
}
