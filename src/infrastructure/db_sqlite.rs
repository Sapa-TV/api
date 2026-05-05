use crate::{
    error::AppResult,
    infrastructure::InitDbData,
    supporters::{SupporterRepository, SupporterRepositoryData},
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::env;
use std::str::FromStr;

mod admin_session;
mod admin_whitelist;
mod models;
mod push_subscriptions;
mod supporters;
mod tokens;

pub struct SqliteDbBuilder {
    pool: SqlitePool,
}

#[derive(Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDbBuilder {
    pub async fn create() -> AppResult<SqliteDb> {
        let db_url = env::var("DATABASE_URL")?;

        tracing::info!("Connecting to database at {}", db_url);

        let connection_options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePool::connect_with(connection_options).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(SqliteDb::new(pool))
    }
}

impl SqliteDb {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init(&self, data: InitDbData) -> AppResult<()> {
        let supporters_data = SupporterRepositoryData {
            king: data.king,
            day_supporters: data.day_supporters,
            month_supporters: data.month_supporters,
        };
        self.init_supporters(supporters_data).await?;
        Ok(())
    }
}
