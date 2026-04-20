use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};

pub const KING_OPTIONS: [&str; 4] = ["Star", "Silent Echo", "Midnight Phantom", "Dragon Lord"];

pub const LAST_DAY_OPTIONS: [&str; 8] = [
    "Star",
    "Echo",
    "Vortex",
    "Night Wolf",
    "Shadow Hunter",
    "Eternal Winter",
    "Crimson Sunrise",
    "Dragon Lord",
];

pub struct InitialData {
    pub king: &'static str,
    pub last_day: Vec<&'static str>,
}

#[derive(Clone)]
pub struct AppState {
    pub king: Arc<RwLock<&'static str>>,
    pub last_day: Arc<RwLock<Vec<&'static str>>>,
}

impl AppState {
    pub fn new(data: InitialData) -> Self {
        Self {
            king: Arc::new(RwLock::new(data.king)),
            last_day: Arc::new(RwLock::new(data.last_day)),
        }
    }
}

async fn get_from_db() -> InitialData {
    InitialData {
        king: KING_OPTIONS[0],
        last_day: vec!["Echo", "Night Wolf", "Shadow Hunter"],
    }
}

pub async fn create_state() -> AppState {
    let data = get_from_db().await;
    AppState::new(data)
}

pub async fn state_updater(state: AppState) {
    let mut king_idx = 0usize;
    let mut last_day_idx = 0usize;
    let mut timer = interval(Duration::from_secs(20));

    loop {
        timer.tick().await;

        {
            let mut king = state.king.write().await;
            *king = KING_OPTIONS[king_idx % KING_OPTIONS.len()];
            king_idx += 1;
        }

        {
            let mut last_day = state.last_day.write().await;
            let new_nick = LAST_DAY_OPTIONS[last_day_idx % LAST_DAY_OPTIONS.len()];
            last_day.push(new_nick);
            if last_day.len() > 10 {
                last_day.remove(0);
            }
            last_day_idx += 1;
        }

        println!("State updated: king and last_day");
    }
}
